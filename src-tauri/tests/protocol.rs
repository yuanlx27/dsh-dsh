//! Immutable native protocol contracts. No WKWebView admission is claimed.
use deepseek_harness_desktop::{
    authentication::BootData,
    bundle::{BundlePaths, Manifest},
    protocol::Protocol,
};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::Path};

fn fixture() -> (tempfile::TempDir, Protocol) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let paths = BundlePaths::staged(root);
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(paths.resources.join("runtime-manifest.json")).unwrap())
            .unwrap();
    let ids = [
        "@deepseek-ai/dsh-client-modules",
        "@deepseek-ai/dsh-client-connection",
    ];
    let entries: Vec<_> = ids.iter().chain(std::iter::once(&"@deepseek-ai/dsh-client-ui-sidebar-terminal")).map(|id| json!({"id":id,"rev":"111111111111","url":format!("plugins/??{id}/client.js&rev=111111111111")})).collect();
    let url = format!(
        "plugins/??{}/client.js,{}/client.js&rev=222222222222",
        ids[0], ids[1]
    );
    let boot = BootData {
        globals: BTreeMap::from([
            (
                "__DSH_BOOT__".into(),
                json!({"rev":"333333333333","entries":entries,"batches":[{"phase":"bootstrap","url":url,"rev":"222222222222","entries":ids}]}),
            ),
            ("__DSH_CONNECTION_RECOVERY__".into(), json!({})),
        ]),
    };
    let temp = tempfile::tempdir().unwrap();
    let shell = BTreeMap::from([(
        "index.html".into(),
        b"<!doctype html><title>Shell fixture</title>".to_vec(),
    )]);
    (
        temp,
        Protocol::new(&paths, &manifest, &boot, shell).unwrap(),
    )
}

#[test]
fn app_shell_assets_are_separate_and_have_no_service_proxy() {
    let (_temp, protocol) = fixture();
    let app = protocol.respond("GET", "dsh-app://app/");
    assert_eq!(app.status, 200);
    assert!(
        String::from_utf8(app.body)
            .unwrap()
            .contains("DeepSeek Harness")
    );
    let shell = protocol.respond("GET", "dsh-app://shell/");
    assert_eq!(shell.status, 200);
    assert!(
        String::from_utf8(shell.body)
            .unwrap()
            .contains("Shell fixture")
    );
    for url in [
        "dsh-app://app/api/remote.mux",
        "dsh-app://app/api/fixture",
        "dsh-app://shell/plugins/??fixture",
        "http://127.0.0.1:1/",
        "dsh-app://other/",
        "dsh-app://user@app/",
    ] {
        assert_ne!(protocol.respond("GET", url).status, 200);
    }
    assert_eq!(protocol.respond("POST", "dsh-app://app/").status, 405);
    assert!(protocol.respond("HEAD", "dsh-app://app/").body.is_empty());
    assert_eq!(
        protocol.respond("GET", "dsh-app://app/missing.css").status,
        404
    );
    let chunk = "dsh-app://app/plugins/@deepseek-ai/dsh-client-ui-sidebar-terminal/client.terminal.js?rev=111111111111";
    assert_eq!(protocol.respond("GET", chunk).status, 200);
    assert_eq!(
        protocol
            .respond("GET", &chunk.replace("111111111111", "999999999999"))
            .status,
        404
    );
    assert_eq!(
        protocol
            .respond(
                "GET",
                &chunk.replace("client.terminal.js", "client.terminal.js.map")
            )
            .status,
        200
    );
}

#[test]
fn traversal_control_and_ambiguous_paths_are_rejected_before_normalization() {
    let (_temp, protocol) = fixture();
    for path in [
        "../index.html",
        "assets/../index.html",
        "%2e%2e/index.html",
        "%252e%252e/index.html",
        "%2findex.html",
        "%5cindex.html",
        "assets\\index.html",
        "%00",
        "%GG",
        "%",
        "/index.html",
        "./index.html",
        "index.html#fragment",
        "index.html?token=fixture",
        "index.html\r\n",
    ] {
        assert_eq!(
            protocol
                .respond("GET", &format!("dsh-app://app/{path}"))
                .status,
            403
        );
    }
}

#[test]
fn combo_order_revision_source_maps_and_private_boot_plan_are_preserved() {
    let (_temp, protocol) = fixture();
    let uri = "dsh-app://app/plugins/??@deepseek-ai/dsh-client-modules/client.js,@deepseek-ai/dsh-client-connection/client.js&rev=222222222222";
    let response = protocol.respond("GET", uri);
    assert_eq!(response.status, 200);
    assert_eq!(response.content_type, "text/javascript; charset=utf-8");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/dsh/desktop-web");
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    let mut expected = Vec::new();
    for id in [
        "@deepseek-ai/dsh-client-modules",
        "@deepseek-ai/dsh-client-connection",
    ] {
        expected.extend(
            fs::read(root.join(index["packages"][id]["main"]["source"].as_str().unwrap())).unwrap(),
        );
    }
    assert!(response.body.starts_with(&expected));
    let text = String::from_utf8(response.body).unwrap();
    assert!(text.ends_with("//# sourceMappingURL=??@deepseek-ai/dsh-client-modules/client.js.map,@deepseek-ai/dsh-client-connection/client.js.map&rev=222222222222\n"));
    assert_eq!(
        protocol
            .respond("GET", &uri.replace("222222222222", "999999999999"))
            .status,
        404
    );
    let map = protocol.respond("GET", &uri.replace("client.js", "client.js.map"));
    assert_eq!(map.status, 200);
    let map: serde_json::Value = serde_json::from_slice(&map.body).unwrap();
    assert_eq!(map["sections"].as_array().unwrap().len(), 2);
    assert!(map["sections"][1]["offset"]["line"].as_u64().unwrap() > 0);
    let plan = serde_json::to_value(protocol.bootstrap()).unwrap();
    assert!(plan["globals"]["__DSH_BOOT__"].is_object());
    assert_eq!(plan["scripts"][0], "dsh-app://app/_desktop/queue.js");
    assert_eq!(plan["scripts"][1], uri);
    // Byte serving never returns per-launch globals; bootstrap admission is T024.
    assert_ne!(
        protocol
            .respond("GET", "dsh-app://app/_desktop/boot.json")
            .status,
        200
    );
}
