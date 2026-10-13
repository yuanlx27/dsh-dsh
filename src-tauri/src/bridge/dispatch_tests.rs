use super::{Operation, tests::fixture};
#[path = "../../tests/transport/fixture.rs"]
mod io;
use crate::dispatch::Dispatcher;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::json;
use std::sync::Arc;
fn handle(value: serde_json::Value) -> String {
    value["handle"].as_str().unwrap().into()
}
#[tokio::test]
async fn service_exit_revokes_auth_and_handles_before_explicit_retry_without_replaying_upload() {
    use crate::{
        app::{Application, NOTICE_REVISION},
        errors::Failure,
        protocol::Protocol,
        runtime::State,
    };
    let io = io::Fixture::new().await;
    let (root, runtime) = crate::runtime::tests::fixture(&format!(
        "echo 'dsh web: {}'\nif [ ! -e exit-used ]; then\nwhile [ ! -e exit-now ]; do sleep 0.02; done\ntouch exit-used\nexit 2\nfi\nIFS= read -r stop\nexit 0",
        io.url
    ));
    let bridge = Arc::new(super::Bridge::new());
    let app = Application::new(
        runtime.clone(),
        bridge.clone(),
        root.path().into(),
        Arc::new(|boot| Ok(Protocol::fixture(boot))),
    )
    .unwrap();
    app.acknowledge(NOTICE_REVISION).unwrap();
    app.start().await.unwrap();
    let first = app.enter().unwrap();
    let binding = bridge.bind(41).unwrap();
    let (_, _, sender, mut request) = fixture();
    request.window_generation = binding.generation;
    request.runtime_generation = first;
    let old_scope = bridge.admit(binding, &sender, &request).unwrap();
    let old_auth = app.session(&old_scope).unwrap();
    let observed = app.clone();
    let store = Dispatcher::new(Arc::new(move |permit| observed.session(permit)));
    let begin = || json!({"action":"begin","path":"/api/upload","method":"POST","headers":[]});
    let upload = handle(
        store
            .call(old_scope.clone(), Operation::Http, begin())
            .await
            .unwrap(),
    );
    store.call(old_scope.clone(), Operation::Http, json!({"action":"write","handle":upload,"sequence":0,"data":STANDARD.encode(b"unsubmitted-exit-fixture")})).await.unwrap();
    let socket = handle(
        store
            .call(
                old_scope.clone(),
                Operation::Stream,
                json!({"action":"begin"}),
            )
            .await
            .unwrap(),
    );
    let mut states = runtime.subscribe();
    std::fs::write(root.path().join("exit-now"), b"exit\n").unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while !matches!(*states.borrow_and_update(), State::Failed(_, _)) {
            states.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(runtime.state(), State::Failed(first, Failure::Exited));
    assert_eq!(
        serde_json::to_value(app.status()).unwrap()["failure"],
        "earlyExit"
    );
    assert!(!old_auth.is_current(first));
    assert!(!old_scope.is_current());
    assert!(app.enter().is_err());
    assert!(app.protocol().is_none());
    assert!(
        store
            .call(
                old_scope.clone(),
                Operation::Http,
                json!({"action":"finish","handle":upload})
            )
            .await
            .is_err()
    );
    assert!(
        store
            .call(
                old_scope.clone(),
                Operation::Stream,
                json!({"action":"read","handle":socket})
            )
            .await
            .is_err()
    );
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(
        std::fs::read_to_string(root.path().join("launch.count"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    assert!(
        io.requests
            .lock()
            .unwrap()
            .iter()
            .all(|r| r.path != "/api/upload")
    );
    // This call represents explicit Retry, not an automatic state observer action.
    app.start().await.unwrap();
    let second = app.enter().unwrap();
    assert!(second > first);
    assert!(root.path().join("exit-used").is_file());
    assert_eq!(
        std::fs::read_to_string(root.path().join("launch.count"))
            .unwrap()
            .lines()
            .count(),
        2
    );
    assert!(bridge.admit(binding, &sender, &request).is_err());
    request.runtime_generation = second;
    let fresh = bridge.admit(binding, &sender, &request).unwrap();
    let fresh_auth = app.session(&fresh).unwrap();
    assert!(!Arc::ptr_eq(&old_auth, &fresh_auth));
    assert!(!old_auth.is_current(second));
    assert!(fresh_auth.is_current(second));
    assert!(
        store
            .call(
                fresh.clone(),
                Operation::Http,
                json!({"action":"finish","handle":upload})
            )
            .await
            .is_err()
    );
    assert!(
        store
            .call(
                fresh.clone(),
                Operation::Stream,
                json!({"action":"read","handle":socket})
            )
            .await
            .is_err()
    );
    assert!(
        io.requests
            .lock()
            .unwrap()
            .iter()
            .all(|r| r.path != "/api/upload")
    );
    let explicit = handle(
        store
            .call(fresh.clone(), Operation::Http, begin())
            .await
            .unwrap(),
    );
    store.call(fresh.clone(), Operation::Http, json!({"action":"write","handle":explicit,"sequence":0,"data":STANDARD.encode(b"new-explicit-fixture")})).await.unwrap();
    store
        .call(
            fresh,
            Operation::Http,
            json!({"action":"finish","handle":explicit}),
        )
        .await
        .unwrap();
    assert_eq!(
        io.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.path == "/api/upload")
            .count(),
        1
    );
    assert_eq!(
        io.requests
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.path == "/api/upload")
            .unwrap()
            .body,
        b"new-explicit-fixture"
    );
    app.shutdown().await.unwrap();
}

#[tokio::test]
async fn scoped_http_handles_reject_other_owners_poison_bad_order_and_preserve_bytes() {
    let f = io::Fixture::new().await;
    let auth = f.authenticate().await.unwrap();
    let store = Dispatcher::new(Arc::new(move |p| {
        if p.is_current() && auth.session.is_current(p.runtime_generation()) {
            Ok(auth.session.clone())
        } else {
            Err(crate::errors::Failure::Unavailable)
        }
    }));
    let (bridge, binding, sender, request) = fixture();
    let scope = bridge.admit(binding, &sender, &request).unwrap();
    let begin = || json!({"action":"begin","path":"/api/upload","method":"POST","headers":[]});
    let key = handle(
        store
            .call(scope.clone(), Operation::Http, begin())
            .await
            .unwrap(),
    );
    let (other, ob, os, or) = fixture();
    let foreign = other.admit(ob, &os, &or).unwrap();
    assert!(
        store
            .call(
                foreign,
                Operation::Http,
                json!({"action":"write","handle":key,"sequence":0,"data":"AA=="})
            )
            .await
            .is_err()
    );
    assert!(
        store
            .call(
                scope.clone(),
                Operation::Http,
                json!({"action":"write","handle":key,"sequence":1,"data":"AA=="})
            )
            .await
            .is_err()
    );
    assert!(
        store
            .call(
                scope.clone(),
                Operation::Http,
                json!({"action":"write","handle":key,"sequence":0,"data":"AA=="})
            )
            .await
            .is_err()
    );
    let key = handle(
        store
            .call(scope.clone(), Operation::Http, begin())
            .await
            .unwrap(),
    );
    let bytes = vec![0, 255, 1, 0];
    store
        .call(
            scope.clone(),
            Operation::Http,
            json!({"action":"write","handle":key,"sequence":0,"data":STANDARD.encode(&bytes)}),
        )
        .await
        .unwrap();
    let head = store
        .call(
            scope.clone(),
            Operation::Http,
            json!({"action":"finish","handle":key}),
        )
        .await
        .unwrap();
    assert_eq!(head["status"], 200);
    assert_eq!(f.requests.lock().unwrap().last().unwrap().body, bytes);
    let chunk = store
        .call(
            scope.clone(),
            Operation::Http,
            json!({"action":"read","handle":key}),
        )
        .await
        .unwrap();
    assert_eq!(chunk["sequence"], 0);
    bridge.detach(binding);
    assert!(
        store
            .call(
                scope,
                Operation::Http,
                json!({"action":"read","handle":key})
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn stream_handles_share_one_scoped_physical_socket_and_do_not_serialize_read_with_write() {
    let f = io::Fixture::new().await;
    let auth = f.authenticate().await.unwrap();
    let store = Dispatcher::new(Arc::new(move |_| Ok(auth.session.clone())));
    let (bridge, binding, sender, request) = fixture();
    let scope = bridge.admit(binding, &sender, &request).unwrap();
    let key = handle(
        store
            .call(scope.clone(), Operation::Stream, json!({"action":"begin"}))
            .await
            .unwrap(),
    );
    assert!(
        store
            .call(scope.clone(), Operation::Stream, json!({"action":"begin"}))
            .await
            .is_err()
    );
    let bytes = "opaque, not a business decoder".repeat(3000).into_bytes();
    store.call(scope.clone(),Operation::Stream,json!({"action":"write","handle":key,"sequence":0,"chunk":0,"data":STANDARD.encode(&bytes[..65536]),"end":false})).await.unwrap();
    let(read,write)=tokio::join!(store.call(scope.clone(),Operation::Stream,json!({"action":"read","handle":key})),store.call(scope.clone(),Operation::Stream,json!({"action":"write","handle":key,"sequence":0,"chunk":1,"data":STANDARD.encode(&bytes[65536..]),"end":true})));
    write.unwrap();
    let first = read.unwrap();
    assert_eq!(first["end"], false);
    assert_eq!(first["sequence"], 0);
    let second = store
        .call(
            scope.clone(),
            Operation::Stream,
            json!({"action":"read","handle":key}),
        )
        .await
        .unwrap();
    assert_eq!(second["end"], true);
    assert_eq!(second["sequence"], 0);
    let mut actual = STANDARD.decode(first["data"].as_str().unwrap()).unwrap();
    actual.extend(STANDARD.decode(second["data"].as_str().unwrap()).unwrap());
    assert_eq!(actual, bytes);
    bridge.detach(binding);
    assert!(
        store
            .call(
                scope,
                Operation::Stream,
                json!({"action":"write","handle":key,"sequence":1,"chunk":0,"data":"","end":true})
            )
            .await
            .is_err()
    );
}
