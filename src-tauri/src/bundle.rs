use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Stdio,
    time::Duration,
};

pub const TARGET: &str = "aarch64-apple-darwin";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    pub desktop_version: String,
    pub dsh_version: String,
    pub upstream_revision: String,
    pub node_version: String,
    pub target: String,
    pub dependency_lock_hash: String,
    pub native_build_number: u64,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
    pub symlink: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBundle;

// Installed Tauri sidecars lose their target suffix. Resource mappings stay fixed.
pub struct BundlePaths {
    pub resources: PathBuf,
    pub binaries: PathBuf,
    pub staged: bool,
}

impl BundlePaths {
    pub fn installed(resources: PathBuf, binaries: PathBuf) -> Self {
        Self {
            resources,
            binaries,
            staged: false,
        }
    }

    pub fn staged(root: &Path) -> Self {
        Self {
            resources: root.join("src-tauri/resources"),
            binaries: root.join("src-tauri/binaries"),
            staged: true,
        }
    }

    pub fn node(&self) -> PathBuf {
        self.binary("node")
    }
    pub fn launcher(&self) -> PathBuf {
        self.binary("dsh")
    }
    pub fn cli(&self) -> PathBuf {
        self.resources
            .join("dsh/node_modules/@deepseek-ai/dsh/lib/bin.js")
    }

    fn binary(&self, name: &str) -> PathBuf {
        self.binaries.join(if self.staged {
            format!("{name}-{TARGET}")
        } else {
            name.into()
        })
    }

    fn artifact_path(&self, path: &str) -> Result<PathBuf, InvalidBundle> {
        if path.contains('\\')
            || path.split('/').any(|part| matches!(part, "" | "." | ".."))
            || !Path::new(path)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
        {
            return Err(InvalidBundle);
        }
        if path == format!("src-tauri/binaries/node-{TARGET}") {
            return Ok(self.node());
        }
        if path == format!("src-tauri/binaries/dsh-{TARGET}") {
            return Ok(self.launcher());
        }
        let relative = path
            .strip_prefix("src-tauri/resources/")
            .ok_or(InvalidBundle)?;
        if !(relative.starts_with("dsh/")
            || matches!(relative, "runtime.lock.json" | "desktop-web.patch.yml"))
        {
            return Err(InvalidBundle);
        }
        Ok(self.resources.join(relative))
    }
}

fn hash_file(path: &Path) -> Result<String, InvalidBundle> {
    let mut file = fs::File::open(path).map_err(|_| InvalidBundle)?;
    let mut hash = Sha256::new();
    let mut bytes = [0_u8; 65536];
    loop {
        let size = file.read(&mut bytes).map_err(|_| InvalidBundle)?;
        if size == 0 {
            break;
        }
        hash.update(&bytes[..size]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn apple_short_version(canonical: &str) -> Result<String, InvalidBundle> {
    let base = canonical.split('-').next().ok_or(InvalidBundle)?;
    let fields: Vec<_> = base.split('.').collect();
    if fields.len() != 3
        || fields
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(InvalidBundle);
    }
    Ok(base.into())
}

fn json(path: &Path) -> Result<serde_json::Value, InvalidBundle> {
    serde_json::from_slice(&fs::read(path).map_err(|_| InvalidBundle)?).map_err(|_| InvalidBundle)
}

fn gather(path: &Path, logical: &str, files: &mut BTreeSet<String>) -> Result<(), InvalidBundle> {
    let metadata = fs::symlink_metadata(path).map_err(|_| InvalidBundle)?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|_| InvalidBundle)? {
            let entry = entry.map_err(|_| InvalidBundle)?;
            let name = entry.file_name().into_string().map_err(|_| InvalidBundle)?;
            gather(&entry.path(), &format!("{logical}/{name}"), files)?;
        }
    } else if metadata.is_file() || metadata.is_symlink() {
        files.insert(logical.into());
    } else {
        return Err(InvalidBundle);
    }
    Ok(())
}

pub async fn verify(paths: &BundlePaths) -> Result<Manifest, InvalidBundle> {
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(paths.resources.join("runtime-manifest.json")).map_err(|_| InvalidBundle)?,
    )
    .map_err(|_| InvalidBundle)?;
    let lock: serde_json::Value =
        serde_json::from_str(include_str!("../../runtime.lock.json")).map_err(|_| InvalidBundle)?;
    let root: serde_json::Value =
        serde_json::from_str(include_str!("../../package.json")).map_err(|_| InvalidBundle)?;
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).map_err(|_| InvalidBundle)?;
    if manifest.desktop_version != env!("CARGO_PKG_VERSION")
        || manifest.dsh_version != manifest.desktop_version
        || root["version"] != manifest.desktop_version
        || config["version"] != manifest.desktop_version
        || lock["desktopVersion"] != manifest.desktop_version
        || lock["dshVersion"] != manifest.dsh_version
        || lock["upstreamRevision"] != manifest.upstream_revision
        || lock["nodeVersion"] != manifest.node_version
        || manifest.target != TARGET
        || lock["target"] != manifest.target
        || lock["dependencyLockHash"] != manifest.dependency_lock_hash
        || lock["nativeBuildNumber"] != manifest.native_build_number
        || manifest.native_build_number == 0
        || config["bundle"]["macOS"]["bundleVersion"].as_str()
            != Some(manifest.native_build_number.to_string().as_str())
        || apple_short_version(&manifest.desktop_version)? != "0.2.0"
        || manifest.artifacts.is_empty()
    {
        return Err(InvalidBundle);
    }
    let package = json(
        &paths
            .resources
            .join("dsh/node_modules/@deepseek-ai/dsh/package.json"),
    )?;
    if package["version"] != manifest.dsh_version {
        return Err(InvalidBundle);
    }
    if hash_file(&paths.resources.join("dsh/package-lock.json"))? != manifest.dependency_lock_hash {
        return Err(InvalidBundle);
    }
    let resolution = json(&paths.resources.join("dsh/package-lock.json"))?;
    for (relative, expected) in resolution["packages"].as_object().ok_or(InvalidBundle)? {
        if relative.is_empty() || expected["dev"] == true {
            continue;
        }
        let supported = |key: &str, value: &str| {
            expected[key].as_array().is_none_or(|items| {
                !items
                    .iter()
                    .any(|v| v.as_str() == Some(&format!("!{value}")))
                    && (items
                        .iter()
                        .all(|v| v.as_str().is_some_and(|s| s.starts_with('!')))
                        || items.iter().any(|v| v.as_str() == Some(value)))
            })
        };
        if !supported("os", "darwin") || !supported("cpu", "arm64") {
            continue;
        }
        if !relative.starts_with("node_modules/")
            || relative.split('/').any(|p| matches!(p, "" | "." | ".."))
        {
            return Err(InvalidBundle);
        }
        let file = paths
            .resources
            .join("dsh")
            .join(relative)
            .join("package.json");
        if !file.try_exists().map_err(|_| InvalidBundle)? && expected["optional"] == true {
            continue;
        }
        if json(&file)?["version"] != expected["version"] {
            return Err(InvalidBundle);
        }
    }
    let immutable_root = paths
        .resources
        .join("dsh")
        .canonicalize()
        .map_err(|_| InvalidBundle)?;
    let mut seen = BTreeSet::new();
    for artifact in &manifest.artifacts {
        if !seen.insert(artifact.path.clone()) {
            return Err(InvalidBundle);
        }
        let path = paths.artifact_path(&artifact.path)?;
        let metadata = fs::symlink_metadata(&path).map_err(|_| InvalidBundle)?;
        let actual = if metadata.is_symlink() {
            let target = fs::read_link(&path).map_err(|_| InvalidBundle)?;
            if target.to_str() != artifact.symlink.as_deref() {
                return Err(InvalidBundle);
            }
            let resolved = path.canonicalize().map_err(|_| InvalidBundle)?;
            if !resolved.starts_with(&immutable_root) || !resolved.is_file() {
                return Err(InvalidBundle);
            }
            format!(
                "{:x}",
                Sha256::digest(target.to_str().ok_or(InvalidBundle)?.as_bytes())
            )
        } else {
            if artifact.symlink.is_some() || !metadata.is_file() {
                return Err(InvalidBundle);
            }
            hash_file(&path)?
        };
        if artifact.sha256 != actual {
            return Err(InvalidBundle);
        }
    }
    let mut actual = BTreeSet::from([
        format!("src-tauri/binaries/node-{TARGET}"),
        format!("src-tauri/binaries/dsh-{TARGET}"),
    ]);
    gather(
        &paths.resources.join("dsh"),
        "src-tauri/resources/dsh",
        &mut actual,
    )?;
    for name in ["runtime.lock.json", "desktop-web.patch.yml"] {
        let path = paths.resources.join(name);
        if path.try_exists().map_err(|_| InvalidBundle)? {
            gather(&path, &format!("src-tauri/resources/{name}"), &mut actual)?;
        }
    }
    if actual != seen
        || !actual.iter().any(|p| p.ends_with(".html"))
        || !actual.iter().any(|p| p.ends_with(".node"))
    {
        return Err(InvalidBundle);
    }
    // Probe only already-hashed, absolute bundled executables, with a disposable home.
    let home = tempfile::tempdir().map_err(|_| InvalidBundle)?;
    for (args, expected) in [
        (
            vec!["--version".into()],
            format!("v{}", manifest.node_version),
        ),
        (vec!["-p".into(), "process.arch".into()], "arm64".into()),
        (
            vec![paths.cli().into_os_string(), "--version".into()],
            manifest.dsh_version.clone(),
        ),
    ] {
        use tokio::io::AsyncReadExt;
        let mut child = tokio::process::Command::new(paths.node())
            .args(args)
            .current_dir(home.path())
            .env("DSH_HOME", home.path())
            .env_remove("NODE_OPTIONS")
            .env_remove("NODE_PATH")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| InvalidBundle)?;
        let output = child.stdout.take().ok_or(InvalidBundle)?;
        let read = async {
            let mut bytes = Vec::new();
            output
                .take(65537)
                .read_to_end(&mut bytes)
                .await
                .map_err(|_| InvalidBundle)?;
            Ok::<_, InvalidBundle>(bytes)
        };
        let wait = async { child.wait().await.map_err(|_| InvalidBundle) };
        let (bytes, status) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::try_join!(read, wait)
        })
        .await
        .map_err(|_| InvalidBundle)??;
        if !status.success()
            || bytes.len() > 65536
            || String::from_utf8(bytes).map_err(|_| InvalidBundle)?.trim() != expected
        {
            return Err(InvalidBundle);
        }
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_identity_and_native_paths_are_distinct() {
        assert_eq!(apple_short_version("0.2.0-rc.2"), Ok("0.2.0".into()));
        assert!(apple_short_version("0.2.invalid").is_err());
        let paths = BundlePaths::installed(
            "/app/Contents/Resources".into(),
            "/app/Contents/MacOS".into(),
        );
        assert_eq!(paths.node(), PathBuf::from("/app/Contents/MacOS/node"));
        assert_eq!(
            paths
                .artifact_path("src-tauri/resources/dsh/lib/bin.js")
                .unwrap(),
            PathBuf::from("/app/Contents/Resources/dsh/lib/bin.js")
        );
        for path in [
            "../outside",
            "src-tauri/resources/dsh/../outside",
            "src-tauri/binaries/unrelated",
            "src-tauri/resources//dsh/bin.js",
        ] {
            assert!(paths.artifact_path(path).is_err());
        }
    }
}
