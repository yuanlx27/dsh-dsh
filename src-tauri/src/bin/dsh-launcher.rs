use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    os::fd::{AsRawFd, FromRawFd},
    path::PathBuf,
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::unix::AsyncFd,
    process::Child,
    signal::unix::{SignalKind, signal},
    time::Instant,
};

fn inputs() -> Result<BTreeMap<String, PathBuf>, ()> {
    let mut values = BTreeMap::new();
    let mut args = std::env::args_os().skip(1);
    while let Some(key) = args.next() {
        let key = key.into_string().map_err(|_| ())?;
        if !["--node", "--cli", "--home", "--cwd", "--overlay"].contains(&key.as_str())
            || values.contains_key(&key)
        {
            return Err(());
        }
        let path = PathBuf::from(args.next().ok_or(())?);
        if !path.is_absolute() {
            return Err(());
        }
        values.insert(key, path);
    }
    if values.len() != 5 {
        return Err(());
    }
    for key in ["--node", "--cli", "--overlay"] {
        if !values[key].is_file() {
            return Err(());
        }
    }
    if !values["--cwd"].is_dir() {
        return Err(());
    }
    Ok(values)
}

async fn owner_stopped() -> io::Result<()> {
    // The private inherited stdin is a pipe, not a terminal or a child input.
    let file = unsafe { File::from_raw_fd(0) };
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    if flags < 0
        || unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
    {
        return Err(io::Error::last_os_error());
    }
    let pipe = AsyncFd::new(file)?;
    loop {
        let mut ready = pipe.readable().await?;
        match ready.try_io(|inner| {
            let mut bytes = [0_u8; 1];
            let mut file = inner.get_ref();
            file.read(&mut bytes)
        }) {
            Ok(result) => return result.map(|_| ()), // Stop byte or owner EOF.
            Err(_) => continue,
        }
    }
}

fn group_signal(group: i32, signal: i32) -> io::Result<()> {
    if group <= 1 {
        return Err(io::Error::other("Invalid owned process group."));
    }
    if unsafe { libc::kill(-group, signal) } == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

fn group_alive(group: i32) -> bool {
    unsafe {
        libc::kill(-group, 0) == 0 || io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
    }
}

async fn cleanup(child: &mut Child, group: i32) -> io::Result<()> {
    group_signal(group, libc::SIGTERM)?;
    let deadline = Instant::now() + Duration::from_secs(7);
    loop {
        child.try_wait()?;
        if !group_alive(group) {
            break;
        }
        if Instant::now() >= deadline {
            group_signal(group, libc::SIGKILL)?;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    // Never disappear while owned survivors remain, even if cleanup is slow.
    while group_alive(group) {
        child.try_wait()?;
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    child.wait().await?;
    Ok(())
}

async fn launch() -> Result<(), ()> {
    let args = inputs()?;
    deepseek_harness_desktop::preferences::ensure_private_dir(&args["--home"]).map_err(|_| ())?;
    // Install signal listeners before spawning the owned group.
    let mut terminate = signal(SignalKind::terminate()).map_err(|_| ())?;
    let mut interrupt = signal(SignalKind::interrupt()).map_err(|_| ())?;
    let mut hangup = signal(SignalKind::hangup()).map_err(|_| ())?;
    let mut child = tokio::process::Command::new(&args["--node"])
        .arg(&args["--cli"])
        .arg("web")
        .arg("--patch")
        .arg(&args["--overlay"])
        .args(["--host", "127.0.0.1", "--port", "0", "--no-open"])
        .env("DSH_HOME", &args["--home"])
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_PATH")
        .current_dir(&args["--cwd"])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .process_group(0)
        .spawn()
        .map_err(|_| ())?;
    let group = i32::try_from(child.id().ok_or(())?).map_err(|_| ())?;
    let unexpected = tokio::select! {
        _ = owner_stopped() => false,
        _ = terminate.recv() => false,
        _ = interrupt.recv() => false,
        _ = hangup.recv() => false,
        _ = child.wait() => true,
    };
    if cleanup(&mut child, group).await.is_err() {
        eprintln!("Owned runtime cleanup failed; the launcher is retained.");
        std::future::pending::<()>().await;
    }
    if unexpected {
        // Exit 2 means the service exited unexpectedly, but its group was cleaned.
        std::process::exit(2);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    if launch().await.is_err() {
        eprintln!("The owned runtime launcher could not complete its operation.");
        std::process::exit(1);
    }
}
