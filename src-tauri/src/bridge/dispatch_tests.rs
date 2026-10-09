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
