//! Immutable bytes only. No service sockets, cookies, general file access or
//! streaming proxy. Bootstrap data is a separate native main-frame handoff.
use crate::{
    authentication::BootData,
    bundle::{BundlePaths, Manifest},
    errors::Failure,
};
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path},
};

#[cfg(test)]
mod tests;

const MAX_ASSET_BYTES: u64 = 64 * 1024 * 1024;
const WEB: &str = "dsh/desktop-web/";
const FRONTEND: &str = "dsh/node_modules/@deepseek-ai/dsh-web-frontend/dist/";

#[derive(Clone)]
pub struct Asset {
    pub status: u16,
    pub content_type: &'static str,
    pub content_length: usize,
    pub body: Vec<u8>,
}
impl Asset {
    fn ok(content_type: &'static str, body: Vec<u8>) -> Self {
        Self {
            status: 200,
            content_type,
            content_length: body.len(),
            body,
        }
    }
    fn empty(status: u16) -> Self {
        Self {
            status,
            content_type: "application/octet-stream",
            content_length: 0,
            body: vec![],
        }
    }
}
#[derive(Serialize)]
pub struct Bootstrap {
    pub globals: BTreeMap<String, Value>,
    pub scripts: Vec<String>,
    pub preloads: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Piece {
    source: String,
    map: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Package {
    main: Piece,
    chunks: BTreeMap<String, Piece>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    version: u32,
    packages: BTreeMap<String, Package>,
}
#[derive(Deserialize)]
struct Entry {
    id: String,
    url: String,
    rev: String,
}
#[derive(Deserialize)]
struct Batch {
    url: String,
    rev: String,
    entries: Vec<String>,
    phase: String,
}
#[derive(Deserialize)]
struct Graph {
    entries: Vec<Entry>,
    batches: Vec<Batch>,
}
struct FrozenPiece {
    source: Vec<u8>,
    map: Value,
}

pub struct Protocol {
    app: BTreeMap<String, Asset>,
    shell: BTreeMap<String, Asset>,
    bootstrap: Bootstrap,
}
impl Protocol {
    // Caller first verifies the bundle. Reads are hash-checked again and cached
    // so later filesystem replacement cannot change already-prepared responses.
    pub fn new(
        paths: &BundlePaths,
        manifest: &Manifest,
        boot: &BootData,
        shell: BTreeMap<String, Vec<u8>>,
    ) -> Result<Self, Failure> {
        let mut app = BTreeMap::new();
        for artifact in &manifest.artifacts {
            if let Some(name) = artifact
                .path
                .strip_prefix(&format!("src-tauri/resources/{FRONTEND}"))
            {
                safe_path(name).map_err(|_| Failure::InvalidBundle)?;
                app.insert(
                    name.into(),
                    Asset::ok(
                        mime(name),
                        read_packaged(paths, manifest, &format!("{FRONTEND}{name}"))?,
                    ),
                );
            }
        }
        let index_bytes = read_packaged(paths, manifest, &format!("{WEB}index.json"))?;
        let index: Index =
            serde_json::from_slice(&index_bytes).map_err(|_| Failure::InvalidBundle)?;
        if index.version != 1 {
            return Err(Failure::InvalidBundle);
        }
        let graph: Graph = serde_json::from_value(
            boot.globals
                .get("__DSH_BOOT__")
                .ok_or(Failure::Connection)?
                .clone(),
        )
        .map_err(|_| Failure::Connection)?;
        let mut pieces = BTreeMap::new();
        let mut revisions = BTreeMap::new();
        for entry in &graph.entries {
            if !revision(&entry.rev)
                || entry.url != combo_url(std::slice::from_ref(&entry.id), &entry.rev, false)
                || revisions
                    .insert(entry.id.clone(), entry.rev.clone())
                    .is_some()
            {
                return Err(Failure::Connection);
            }
            let package = index
                .packages
                .get(&entry.id)
                .ok_or(Failure::InvalidBundle)?;
            pieces.insert(
                entry.id.clone(),
                read_piece(paths, manifest, &package.main)?,
            );
            for (name, piece) in &package.chunks {
                if !chunk_name(name) {
                    return Err(Failure::InvalidBundle);
                }
                let piece = read_piece(paths, manifest, piece)?;
                let url = format!("plugins/{}/{name}?rev={}", entry.id, entry.rev);
                let mut source = piece.source.clone();
                source.extend_from_slice(
                    format!("//# sourceMappingURL={name}.map?rev={}\n", entry.rev).as_bytes(),
                );
                app.insert(url, Asset::ok("text/javascript; charset=utf-8", source));
                app.insert(
                    format!("plugins/{}/{name}.map?rev={}", entry.id, entry.rev),
                    source_map(name, &[&piece])?,
                );
            }
        }
        if !pieces.contains_key("@deepseek-ai/dsh-client-modules") {
            return Err(Failure::Connection);
        }
        for entry in &graph.entries {
            insert_combo(
                &mut app,
                std::slice::from_ref(&entry.id),
                &entry.rev,
                &pieces,
            )?;
        }
        let mut bootstrap = Bootstrap {
            globals: boot.globals.clone(),
            scripts: vec!["dsh-app://app/_desktop/queue.js".into()],
            preloads: vec![],
        };
        let mut batch_urls = BTreeMap::new();
        for batch in &graph.batches {
            if batch.entries.is_empty()
                || !revision(&batch.rev)
                || batch.url != combo_url(&batch.entries, &batch.rev, false)
                || batch_urls.insert(batch.url.clone(), ()).is_some()
            {
                return Err(Failure::Connection);
            }
            insert_combo(&mut app, &batch.entries, &batch.rev, &pieces)?;
            let url = format!("dsh-app://app/{}", batch.url);
            match batch.phase.as_str() {
                "bootstrap" => bootstrap.scripts.push(url),
                "application" => bootstrap.preloads.push(url),
                _ => return Err(Failure::Connection),
            }
        }
        if bootstrap.scripts.len() < 2 {
            return Err(Failure::Connection);
        }
        app.insert(
            "_desktop/queue.js".into(),
            Asset::ok(
                "text/javascript; charset=utf-8",
                read_packaged(paths, manifest, &format!("{WEB}queue.js"))?,
            ),
        );
        let index = app.get_mut("index.html").ok_or(Failure::InvalidBundle)?;
        let html = std::str::from_utf8(&index.body).map_err(|_| Failure::InvalidBundle)?;
        if !html.contains("<head>") {
            return Err(Failure::InvalidBundle);
        }
        // No inline program or per-launch globals in the byte responder.
        index.body = html
            .replacen(
                "<head>",
                "<head><script src=\"./_desktop/queue.js\"></script>",
                1,
            )
            .into_bytes();
        index.content_length = index.body.len();
        let mut shell_assets = BTreeMap::new();
        for (name, body) in shell {
            safe_path(&name).map_err(|_| Failure::InvalidBundle)?;
            shell_assets.insert(name.clone(), Asset::ok(mime(&name), body));
        }
        Ok(Self {
            app,
            shell: shell_assets,
            bootstrap,
        })
    }
    // Only T024's admitted native main-frame bootstrap caller may return this.
    // It is deliberately not a public URI endpoint or a renderer command here.
    pub fn bootstrap(&self) -> &Bootstrap {
        &self.bootstrap
    }

    pub fn respond(&self, method: &str, uri: &str) -> Asset {
        if !matches!(method, "GET" | "HEAD") {
            return Asset::empty(405);
        }
        let (assets, target) = if let Some(path) = uri.strip_prefix("dsh-app://app/") {
            (&self.app, path)
        } else if let Some(path) = uri.strip_prefix("dsh-app://shell/") {
            (&self.shell, path)
        } else {
            return Asset::empty(403);
        };
        if target.contains('#') || target.chars().any(char::is_control) {
            return Asset::empty(403);
        }
        let (path, query) = target
            .split_once('?')
            .map_or((target, None), |(p, q)| (p, Some(q)));
        let Ok(path) = safe_path(path) else {
            return Asset::empty(403);
        };
        if query.is_some_and(|q| {
            url::form_urlencoded::parse(q.as_bytes()).any(|(name, _)| name == "token")
        }) {
            return Asset::empty(403);
        }
        let key = match query {
            Some(q) if path.starts_with("plugins/") => format!("{path}?{q}"),
            Some(_) => return Asset::empty(403),
            None => {
                if path.is_empty() {
                    "index.html".into()
                } else {
                    path
                }
            }
        };
        let mut response = assets
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Asset::empty(404));
        if method == "HEAD" {
            response.body.clear();
        }
        response
    }
}

fn safe_path(raw: &str) -> Result<String, ()> {
    let bytes = raw.as_bytes();
    for (at, byte) in bytes.iter().enumerate() {
        if *byte == b'%'
            && (at + 2 >= bytes.len()
                || !bytes[at + 1].is_ascii_hexdigit()
                || !bytes[at + 2].is_ascii_hexdigit())
        {
            return Err(());
        }
    }
    let lower = raw.to_ascii_lowercase();
    if lower.contains("%2f") || lower.contains("%5c") {
        return Err(());
    }
    let path = percent_decode_str(raw)
        .decode_utf8()
        .map_err(|_| ())?
        .into_owned();
    if path.starts_with('/')
        || path.contains(['%', '\\'])
        || path.chars().any(char::is_control)
        || path.split('/').any(|p| matches!(p, "." | ".."))
        || path.split('/').any(|p| p.is_empty()) && !path.is_empty() && path != "plugins/"
    {
        return Err(());
    }
    Ok(path)
}
fn revision(value: &str) -> bool {
    value.len() == 12
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn chunk_name(value: &str) -> bool {
    value
        .strip_prefix("client.")
        .and_then(|v| v.strip_suffix(".js"))
        .is_some_and(|v| {
            !v.is_empty()
                && v.bytes().enumerate().all(|(i, b)| {
                    b.is_ascii_alphanumeric() || i > 0 && matches!(b, b'.' | b'_' | b'-')
                })
        })
}
fn combo_url(ids: &[String], rev: &str, map: bool) -> String {
    format!(
        "plugins/??{}&rev={rev}",
        ids.iter()
            .map(|id| format!("{id}/client.js{}", if map { ".map" } else { "" }))
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn read_piece(
    paths: &BundlePaths,
    manifest: &Manifest,
    piece: &Piece,
) -> Result<FrozenPiece, Failure> {
    safe_path(&piece.source).map_err(|_| Failure::InvalidBundle)?;
    safe_path(&piece.map).map_err(|_| Failure::InvalidBundle)?;
    Ok(FrozenPiece {
        source: read_packaged(paths, manifest, &format!("{WEB}{}", piece.source))?,
        map: serde_json::from_slice(&read_packaged(
            paths,
            manifest,
            &format!("{WEB}{}", piece.map),
        )?)
        .map_err(|_| Failure::InvalidBundle)?,
    })
}
fn insert_combo(
    app: &mut BTreeMap<String, Asset>,
    ids: &[String],
    rev: &str,
    pieces: &BTreeMap<String, FrozenPiece>,
) -> Result<(), Failure> {
    let selected: Vec<_> = ids
        .iter()
        .map(|id| pieces.get(id).ok_or(Failure::Connection))
        .collect::<Result<_, _>>()?;
    let mut source = Vec::new();
    for piece in &selected {
        if source.len() + piece.source.len() > MAX_ASSET_BYTES as usize {
            return Err(Failure::InvalidBundle);
        }
        source.extend_from_slice(&piece.source);
    }
    let map_url = combo_url(ids, rev, true);
    source.extend_from_slice(
        format!(
            "//# sourceMappingURL={}\n",
            map_url
                .strip_prefix("plugins/")
                .ok_or(Failure::Connection)?
        )
        .as_bytes(),
    );
    app.insert(
        combo_url(ids, rev, false),
        Asset::ok("text/javascript; charset=utf-8", source),
    );
    app.insert(map_url, source_map("client.js", &selected)?);
    Ok(())
}
fn source_map(file: &str, pieces: &[&FrozenPiece]) -> Result<Asset, Failure> {
    let mut line = 0;
    let mut sections = vec![];
    for piece in pieces {
        sections.push(json!({"offset":{"line":line,"column":0},"map":piece.map}));
        line += piece.source.iter().filter(|b| **b == b'\n').count();
    }
    let mut bytes = serde_json::to_vec(&json!({"version":3,"file":file,"sections":sections}))
        .map_err(|_| Failure::InvalidBundle)?;
    bytes.push(b'\n');
    if bytes.len() as u64 > MAX_ASSET_BYTES {
        return Err(Failure::InvalidBundle);
    }
    Ok(Asset::ok("application/json; charset=utf-8", bytes))
}
fn read_packaged(
    paths: &BundlePaths,
    manifest: &Manifest,
    relative: &str,
) -> Result<Vec<u8>, Failure> {
    if !Path::new(relative)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(Failure::InvalidBundle);
    }
    let root = paths
        .resources
        .canonicalize()
        .map_err(|_| Failure::InvalidBundle)?;
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|_| Failure::InvalidBundle)?;
    if !path.starts_with(root.join("dsh")) {
        return Err(Failure::InvalidBundle);
    }
    let resolved = path
        .strip_prefix(&root)
        .map_err(|_| Failure::InvalidBundle)?
        .to_str()
        .ok_or(Failure::InvalidBundle)?;
    let artifact = manifest
        .artifacts
        .iter()
        .find(|a| a.path == format!("src-tauri/resources/{resolved}") && a.symlink.is_none())
        .ok_or(Failure::InvalidBundle)?;
    if fs::metadata(&path)
        .map_err(|_| Failure::InvalidBundle)?
        .len()
        > MAX_ASSET_BYTES
    {
        return Err(Failure::InvalidBundle);
    }
    let mut body = Vec::new();
    fs::File::open(path)
        .map_err(|_| Failure::InvalidBundle)?
        .take(MAX_ASSET_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|_| Failure::InvalidBundle)?;
    if body.len() as u64 > MAX_ASSET_BYTES {
        return Err(Failure::InvalidBundle);
    }
    if format!("{:x}", Sha256::digest(&body)) != artifact.sha256 {
        return Err(Failure::InvalidBundle);
    }
    Ok(body)
}
fn mime(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|x| x.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map" | "webmanifest") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
