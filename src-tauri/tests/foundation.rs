use deepseek_harness_desktop::{preferences, runtime};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
fn private_preferences_roundtrip_and_schema_rejection() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("private app data");
    let prefs = preferences::Preferences {
        schema_version: 1,
        safety_notice_revision: Some("notice-1".into()),
        window_bounds: None,
    };
    preferences::save(&root, &prefs).unwrap();
    assert_eq!(preferences::load(&root).unwrap(), prefs);
    assert_eq!(
        fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let file = root.join("preferences.json");
    assert_eq!(
        fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::write(&file, r#"{"schemaVersion":2}"#).unwrap();
    assert!(preferences::load(&root).is_err());
    fs::write(&file, r#"{"schemaVersion":1,"credential":"forbidden"}"#).unwrap();
    assert!(preferences::load(&root).is_err());
    fs::write(&file, "{").unwrap();
    assert!(preferences::load(&root).is_err());
}

#[test]
fn failed_atomic_replacement_preserves_previous_preferences() {
    let temp = tempfile::tempdir().unwrap();
    let prefs = preferences::Preferences::default();
    preferences::save(temp.path(), &prefs).unwrap();
    let path = temp.path().join("preferences.json");
    let original = fs::read(&path).unwrap();
    let mut invalid = prefs.clone();
    invalid.schema_version = 2;
    assert!(preferences::save(temp.path(), &invalid).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(preferences::save(temp.path(), &prefs).is_err());
    assert!(path.is_dir());
}

#[test]
fn bounds_must_fit_a_current_display() {
    use preferences::Bounds;
    let screen = Bounds {
        x: 0.0,
        y: 0.0,
        width: 1440.0,
        height: 900.0,
    };
    let usable = Bounds {
        x: 100.0,
        y: 50.0,
        width: 800.0,
        height: 600.0,
    };
    assert_eq!(
        preferences::usable_bounds(Some(usable), &[screen]),
        Some(usable)
    );
    for bounds in [
        Bounds {
            x: -2000.0,
            ..usable
        },
        Bounds {
            width: 0.0,
            ..usable
        },
        Bounds {
            width: f64::NAN,
            ..usable
        },
        Bounds {
            height: 2000.0,
            ..usable
        },
    ] {
        assert_eq!(preferences::usable_bounds(Some(bounds), &[screen]), None);
    }
    assert_eq!(preferences::usable_bounds(Some(usable), &[]), None);
}

#[test]
fn parser_only_accepts_the_owned_loopback_auth_announcement() {
    assert!(
        runtime::parse_readiness("bounded unrelated log")
            .unwrap()
            .is_none()
    );
    let url = runtime::parse_readiness("dsh web: http://127.0.0.1:54321/?token=fixture")
        .unwrap()
        .unwrap();
    assert_eq!(url.port(), Some(54321));
    assert_eq!(url.query(), Some("token=fixture"));
    for line in [
        "dsh web: https://127.0.0.1:54321/?token=fixture",
        "dsh web: http://localhost:54321/?token=fixture",
        "dsh web: http://192.168.1.2:54321/?token=fixture",
        "dsh web: http://127.0.0.1:0/?token=fixture",
        "dsh web: http://user:pass@127.0.0.1:54321/?token=fixture",
        "dsh web: http://127.0.0.1:54321/?token=fixture (LAN: http://192.168.1.2:54321)",
        "dsh web: http://127.0.0.1:54321/other?token=fixture",
        "dsh web: http://127.0.0.1:54321/",
        "dsh web: not-a-url",
    ] {
        let error = runtime::parse_readiness(line).unwrap_err();
        assert!(!format!("{error:?}").contains("fixture"));
    }
}

#[tokio::test]
async fn bounded_readiness_timeout_and_early_exit() {
    use tokio::io::{AsyncWriteExt, BufReader};
    let input = b"unrelated\ndsh web: http://127.0.0.1:54321/?token=fixture\n";
    let ready = runtime::read_readiness(
        BufReader::new(&input[..]),
        tokio::time::Instant::now() + Duration::from_secs(1),
    )
    .await
    .unwrap();
    assert_eq!(ready.port(), Some(54321));
    for input in [
        vec![b'x'; 65537],
        vec![0xff, b'\n'],
        b"no announcement\n".to_vec(),
    ] {
        assert!(
            runtime::read_readiness(
                BufReader::new(&input[..]),
                tokio::time::Instant::now() + Duration::from_secs(1)
            )
            .await
            .is_err()
        );
    }
    let (reader, mut writer) = tokio::io::duplex(64);
    writer.write_all(b"unfinished").await.unwrap();
    let start = Instant::now();
    assert!(
        runtime::read_readiness(
            BufReader::new(reader),
            tokio::time::Instant::now() + Duration::from_millis(30)
        )
        .await
        .is_err()
    );
    assert!(start.elapsed() < Duration::from_secs(1));
}

fn wait_for_exit(child: &mut std::process::Child) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("Owned launcher did not finish cleanup.");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn owned_launcher(dir: &Path) -> std::process::Child {
    let cli = dir.join("fixture-service.sh");
    fs::write(
        &cli,
        "echo helper-ready\ntrap 'exit 0' TERM INT\nwhile :; do sleep 1; done\n",
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_dsh-launcher"))
        .args(["--node", "/bin/sh", "--cli"])
        .arg(cli)
        .arg("--home")
        .arg(dir.join("dsh-home"))
        .arg("--cwd")
        .arg(dir)
        .arg("--overlay")
        .arg(dir.join("fixture.patch.yml"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

#[test]
fn owner_pipe_eof_cleans_only_the_owned_processes() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("fixture.patch.yml"), "{}").unwrap();
    let mut sentinel = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let mut launcher = owned_launcher(temp.path());
    let mut bytes = [0_u8; 13];
    launcher
        .stdout
        .take()
        .unwrap()
        .read_exact(&mut bytes)
        .unwrap();
    assert_eq!(&bytes, b"helper-ready\n");
    drop(launcher.stdin.take());
    wait_for_exit(&mut launcher);
    assert!(sentinel.try_wait().unwrap().is_none());
    sentinel.kill().unwrap();
    sentinel.wait().unwrap();
}

#[test]
fn explicit_stop_waits_for_owned_exit() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("fixture.patch.yml"), "{}").unwrap();
    let mut launcher = owned_launcher(temp.path());
    let mut bytes = [0_u8; 13];
    launcher
        .stdout
        .take()
        .unwrap()
        .read_exact(&mut bytes)
        .unwrap();
    launcher
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"stop\n")
        .unwrap();
    wait_for_exit(&mut launcher);
}
