use super::tests::fixture;
#[path = "../../tests/transport/fixture.rs"]
mod io;
use crate::http_transport::{Body, HttpRequest, Transport};
use reqwest::{Method, header::HeaderMap};
use tokio::sync::watch;

#[tokio::test]
async fn admitted_http_denies_revoked_window_before_dispatch_and_during_headers() {
    let f = io::Fixture::new().await;
    let auth = f.authenticate().await.unwrap();
    let transport = Transport::new(auth.session);
    let (bridge, binding, sender, request) = fixture();
    let permit = bridge.admit(binding, &sender, &request).unwrap();
    let (_cancel, signal) = watch::channel(false);
    let input = || HttpRequest {
        path: "/api/fixture".into(),
        method: Method::GET,
        headers: HeaderMap::new(),
        body: Body::default(),
    };
    let mut reply = transport
        .fetch_admitted(&permit, input(), signal.clone())
        .await
        .unwrap();
    f.api_entered.notified().await; // Consume the first request's notification.
    bridge.detach(binding);
    assert!(reply.next_chunk().await.is_err());
    assert!(
        transport
            .fetch_admitted(&permit, input(), signal.clone())
            .await
            .is_err()
    );
    assert_eq!(f.count(), 3);
    let binding = bridge.bind(41).unwrap();
    let request = super::Request {
        window_generation: binding.generation,
        ..request
    };
    let permit = bridge.admit(binding, &sender, &request).unwrap();
    f.api.lock().unwrap().hold = true;
    let fetch = transport.fetch_admitted(&permit, input(), signal);
    tokio::pin!(fetch);
    tokio::select! {
        _ = f.api_entered.notified() => {},
        _ = &mut fetch => panic!("Held headers completed."),
    }
    bridge.detach(binding);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(1), fetch)
            .await
            .unwrap()
            .is_err()
    );
    assert_eq!(f.count(), 4);
}
