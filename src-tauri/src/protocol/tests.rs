use super::*;
use crate::bundle::Artifact;

fn fixture() -> (tempfile::TempDir, BundlePaths, Manifest) {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("dsh")).unwrap();
    fs::write(temp.path().join("dsh/asset.js"), b"fixture bytes").unwrap();
    let paths = BundlePaths::installed(temp.path().into(), temp.path().into());
    let manifest = Manifest {
        desktop_version: "0.2.0-rc.2".into(),
        dsh_version: "0.2.0-rc.2".into(),
        upstream_revision: String::new(),
        node_version: "24.21.0".into(),
        target: String::new(),
        dependency_lock_hash: String::new(),
        native_build_number: 1,
        artifacts: vec![Artifact {
            path: "src-tauri/resources/dsh/asset.js".into(),
            sha256: format!("{:x}", Sha256::digest(b"fixture bytes")),
            symlink: None,
        }],
    };
    (temp, paths, manifest)
}
#[test]
fn packaged_reads_reject_corruption_unlisted_files_and_external_symlinks() {
    let (temp, paths, manifest) = fixture();
    assert_eq!(
        read_packaged(&paths, &manifest, "dsh/asset.js").unwrap(),
        b"fixture bytes"
    );
    assert!(read_packaged(&paths, &manifest, "dsh/../asset.js").is_err());
    fs::write(temp.path().join("dsh/private.json"), b"not inventoried").unwrap();
    assert!(read_packaged(&paths, &manifest, "dsh/private.json").is_err());
    fs::write(temp.path().join("dsh/asset.js"), b"tampered").unwrap();
    assert!(read_packaged(&paths, &manifest, "dsh/asset.js").is_err());
    fs::remove_file(temp.path().join("dsh/asset.js")).unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::os::unix::fs::symlink(outside.path(), temp.path().join("dsh/asset.js")).unwrap();
    assert!(read_packaged(&paths, &manifest, "dsh/asset.js").is_err());
}
#[test]
fn internal_symlink_uses_target_inventory_digest_and_cached_reply_is_immutable() {
    let (temp, paths, manifest) = fixture();
    std::os::unix::fs::symlink("asset.js", temp.path().join("dsh/link.js")).unwrap();
    let bytes = read_packaged(&paths, &manifest, "dsh/link.js").unwrap();
    let protocol = Protocol {
        app: BTreeMap::from([(
            "index.html".into(),
            Asset::ok("text/html; charset=utf-8", bytes.clone()),
        )]),
        shell: BTreeMap::new(),
        bootstrap: Bootstrap {
            globals: BTreeMap::new(),
            scripts: vec![],
            preloads: vec![],
        },
    };
    fs::write(temp.path().join("dsh/asset.js"), b"replacement").unwrap();
    assert_eq!(protocol.respond("GET", "dsh-app://app/").body, bytes);
    let head = protocol.respond("HEAD", "dsh-app://app/");
    assert_eq!(head.content_length, bytes.len());
    assert!(head.body.is_empty());
    let mut first = protocol.respond("GET", "dsh-app://app/");
    first.body.clear();
    assert_eq!(protocol.respond("GET", "dsh-app://app/").body, bytes);
}
#[test]
fn package_chunk_grammar_and_unicode_decode_are_exact() {
    for good in ["client.a.js", "client.0-a_b.c.js"] {
        assert!(chunk_name(good));
    }
    for bad in [
        "client.js",
        "client..js",
        "client._a.js",
        "client.a/b.js",
        "client.%2e.js",
    ] {
        assert!(!chunk_name(bad));
    }
    assert_eq!(safe_path("icons/%E4%B8%AD.svg"), Ok("icons/中.svg".into()));
    assert!(safe_path("%FF").is_err());
}
