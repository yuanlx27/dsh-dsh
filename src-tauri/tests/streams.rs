#[path = "transport/fixture.rs"]
mod fixture;
use deepseek_harness_desktop::stream_transport::{Frame, StreamTransport};
use fixture::Fixture;
use tokio::sync::watch;

#[tokio::test]
async fn owned_ws_has_native_auth_and_preserves_opaque_text_binary_order() {
    let f = Fixture::new().await;
    let auth = f.authenticate().await.unwrap();
    let transport = StreamTransport::new(auth.session);
    let (_cancel, signal) = watch::channel(false);
    let mut stream = transport.connect(7, signal).await.unwrap();
    let uplink = stream.sender();
    let first = tokio::join!(stream.next_frame(), async {
        uplink
            .send(0, Frame::Text("while read is pending".into()))
            .await
    });
    assert!(first.1.is_ok());
    assert_eq!(
        first.0.unwrap().unwrap().frame,
        Frame::Text("while read is pending".into())
    );
    let text = Frame::Text(
        r#"{"type":"open","streamId":"opaque","endpoint":"fixture","payload":{"same":[1,1]}}"#
            .into(),
    );
    let binary = Frame::Binary(vec![0, 255, 0, 1]);
    stream.send(1, text.clone()).await.unwrap();
    stream.send(2, binary.clone()).await.unwrap();
    assert_eq!(stream.next_frame().await.unwrap().unwrap().frame, text);
    assert_eq!(stream.next_frame().await.unwrap().unwrap().frame, binary);
    let headers = f.ws_headers.lock().unwrap();
    assert_eq!(headers["cookie"], f.cookie);
    assert_eq!(headers["origin"], f.url.origin().ascii_serialization());
}
#[tokio::test]
async fn sequence_cancellation_and_auth_revocation_are_terminal_without_reconnect() {
    let f = Fixture::new().await;
    let auth = f.authenticate().await.unwrap();
    let transport = StreamTransport::new(auth.session.clone());
    let (cancel, signal) = watch::channel(false);
    let mut stream = transport.connect(7, signal).await.unwrap();
    assert!(
        stream
            .send(1, Frame::Text("out of order".into()))
            .await
            .is_err()
    );
    assert!(stream.send(0, Frame::Text("late".into())).await.is_err());
    cancel.send(true).unwrap();
    assert!(stream.next_frame().await.is_err());
    auth.session.invalidate();
    let (_cancel, signal) = watch::channel(false);
    assert!(transport.connect(7, signal).await.is_err());
}
