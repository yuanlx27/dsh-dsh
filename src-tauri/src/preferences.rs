use serde::{Deserialize, Serialize};
use std::{
    fs::{self, DirBuilder, Permissions},
    io::{self, Read, Write},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub schema_version: u32,
    pub safety_notice_revision: Option<String>,
    pub window_bounds: Option<Bounds>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 1,
            safety_notice_revision: None,
            window_bounds: None,
        }
    }
}

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Application preferences are unsupported or invalid.",
    )
}

pub fn ensure_private_dir(root: &Path) -> io::Result<()> {
    DirBuilder::new().recursive(true).mode(0o700).create(root)?;
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err(invalid());
    }
    fs::set_permissions(root, Permissions::from_mode(0o700))
}

pub fn load(root: &Path) -> io::Result<Preferences> {
    let path = root.join("preferences.json");
    let metadata = match fs::symlink_metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Preferences::default()),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || metadata.len() > 65536 || metadata.permissions().mode() & 0o077 != 0 {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(invalid());
    }
    let prefs: Preferences = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if prefs.schema_version != 1 {
        return Err(invalid());
    }
    Ok(prefs)
}

pub fn save(root: &Path, prefs: &Preferences) -> io::Result<()> {
    if prefs.schema_version != 1
        || prefs
            .safety_notice_revision
            .as_ref()
            .is_some_and(|s| s.is_empty() || s.len() > 1024)
        || prefs.window_bounds.is_some_and(|b| !finite(b))
    {
        return Err(invalid());
    }
    ensure_private_dir(root)?;
    // Refuse unknown existing schemas instead of silently overwriting them.
    load(root)?;
    let mut temp = tempfile::NamedTempFile::new_in(root)?;
    temp.as_file()
        .set_permissions(Permissions::from_mode(0o600))?;
    serde_json::to_writer(temp.as_file_mut(), prefs).map_err(|_| invalid())?;
    temp.write_all(b"\n")?;
    temp.as_file().sync_all()?;
    temp.persist(root.join("preferences.json"))
        .map_err(|error| error.error)?;
    fs::File::open(root)?.sync_all()
}

fn finite(bounds: Bounds) -> bool {
    [bounds.x, bounds.y, bounds.width, bounds.height]
        .iter()
        .all(|value| value.is_finite())
}

pub fn usable_bounds(bounds: Option<Bounds>, screens: &[Bounds]) -> Option<Bounds> {
    bounds.filter(|b| {
        finite(*b)
            && b.width >= 320.0
            && b.height >= 240.0
            && screens.iter().any(|s| {
                finite(*s)
                    && b.x >= s.x
                    && b.y >= s.y
                    && b.x + b.width <= s.x + s.width
                    && b.y + b.height <= s.y + s.height
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_private_roundtrip_and_unsupported_schema() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("preferences with spaces");
        let prefs = Preferences {
            safety_notice_revision: Some("notice-1".into()),
            ..Preferences::default()
        };
        save(&root, &prefs).unwrap();
        assert_eq!(load(&root).unwrap(), prefs);
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        let path = root.join("preferences.json");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::write(&path, r#"{"schemaVersion":2}"#).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(load(&root).is_err());
        assert!(save(&root, &prefs).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
    }

    #[test]
    fn replacement_failure_and_unusable_bounds_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("preferences.json")).unwrap();
        assert!(save(temp.path(), &Preferences::default()).is_err());
        let screen = Bounds {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        };
        assert_eq!(usable_bounds(Some(screen), &[screen]), Some(screen));
        assert_eq!(
            usable_bounds(
                Some(Bounds {
                    x: -2000.0,
                    ..screen
                }),
                &[screen]
            ),
            None
        );
        assert_eq!(
            usable_bounds(
                Some(Bounds {
                    width: f64::NAN,
                    ..screen
                }),
                &[screen]
            ),
            None
        );
    }
}
