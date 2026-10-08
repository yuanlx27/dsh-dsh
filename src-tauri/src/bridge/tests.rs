//! Native policy fixtures, not proof of installed WK metadata behavior.
use super::*;
fn fixture() -> (Arc<Bridge>, Binding, Sender, Request) {
    let bridge = Arc::new(Bridge::new());
    let binding = bridge.bind(41).unwrap();
    bridge.set_runtime(7, true, true).unwrap();
    let sender = Sender {
        message_view: 41,
        frame_view: 41,
        main_frame: true,
        protocol: "dsh-app".into(),
        host: "app".into(),
        port: 0,
        frame_url: Some("dsh-app://app/".into()),
        view_url: Some("dsh-app://app/".into()),
    };
    let request = Request {
        window_generation: binding.generation,
        runtime_generation: 7,
        operation: Operation::Bootstrap,
        payload: serde_json::Value::Null,
    };
    (bridge, binding, sender, request)
}
#[test]
fn permits_from_different_native_owners_cannot_address_each_others_handles() {
    let (left, lb, ls, lr) = fixture();
    let (right, rb, rs, rr) = fixture();
    let lp = left.admit(lb, &ls, &lr).unwrap();
    let rp = right.admit(rb, &rs, &rr).unwrap();
    assert!(lp.same_scope(&lp.clone()));
    assert!(!lp.same_scope(&rp));
    left.detach(lb);
    assert!(!lp.is_current());
    assert!(rp.is_current());
}
#[test]
fn only_owned_actual_app_main_frame_is_admitted() {
    let (bridge, binding, sender, request) = fixture();
    assert!(
        bridge
            .admit(binding, &sender, &request)
            .unwrap()
            .is_current()
    );
    let variants = [
        Sender {
            main_frame: false,
            ..sender.clone()
        },
        Sender {
            message_view: 42,
            ..sender.clone()
        },
        Sender {
            frame_view: 42,
            ..sender.clone()
        },
        Sender {
            message_view: 0,
            ..sender.clone()
        },
        Sender {
            host: "shell".into(),
            ..sender.clone()
        },
        Sender {
            host: "other".into(),
            ..sender.clone()
        },
        Sender {
            protocol: "https".into(),
            ..sender.clone()
        },
        Sender {
            port: 1,
            ..sender.clone()
        },
        Sender {
            frame_url: Some("https://example.invalid/".into()),
            ..sender.clone()
        },
        Sender {
            frame_url: Some("dsh-app://user@app/".into()),
            ..sender.clone()
        },
        Sender {
            view_url: Some("dsh-app://shell/".into()),
            ..sender.clone()
        },
        Sender {
            frame_url: None,
            ..sender.clone()
        },
    ];
    for bad in variants {
        assert!(bridge.admit(binding, &bad, &request).is_err());
    }
}
#[test]
fn bootstrap_http_and_stream_require_both_native_ready_and_saved_notice() {
    let (bridge, binding, sender, mut request) = fixture();
    for operation in [Operation::Bootstrap, Operation::Http, Operation::Stream] {
        request.operation = operation;
        for (ready, notice) in [(false, false), (false, true), (true, false)] {
            request.runtime_generation += 1;
            bridge
                .set_runtime(request.runtime_generation, ready, notice)
                .unwrap();
            assert!(bridge.admit(binding, &sender, &request).is_err());
        }
        request.runtime_generation += 1;
        bridge
            .set_runtime(request.runtime_generation, true, true)
            .unwrap();
        assert!(bridge.admit(binding, &sender, &request).is_ok());
    }
}
#[tokio::test]
async fn runtime_window_and_gate_revocation_cancel_old_permits_and_replies() {
    let (bridge, binding, sender, mut request) = fixture();
    let permit = bridge.admit(binding, &sender, &request).unwrap();
    bridge.set_runtime(8, true, true).unwrap();
    assert!(!permit.is_current());
    let mut delivered = false;
    assert!(
        permit
            .with_current(|| {
                delivered = true;
            })
            .is_err()
    );
    assert!(!delivered);
    tokio::time::timeout(std::time::Duration::from_secs(1), permit.revoked())
        .await
        .unwrap();
    assert!(bridge.admit(binding, &sender, &request).is_err());
    request.runtime_generation = 8;
    let current = bridge.admit(binding, &sender, &request).unwrap();
    let replacement = bridge.bind(41).unwrap(); // Same pointer after reload still has a new native generation.
    assert!(!current.is_current());
    assert!(bridge.admit(binding, &sender, &request).is_err());
    assert!(bridge.admit(replacement, &sender, &request).is_err());
    request.window_generation = replacement.generation;
    let new = bridge.admit(replacement, &sender, &request).unwrap();
    bridge.detach(binding); // A late old disposal cannot revoke the replacement.
    assert!(new.is_current());
    bridge.set_runtime(8, true, false).unwrap();
    assert!(!new.is_current());
}
#[test]
fn runtime_activation_is_monotonic_and_cannot_resurrect_a_failed_generation() {
    let (bridge, binding, sender, request) = fixture();
    bridge.set_runtime(7, false, true).unwrap();
    assert!(bridge.set_runtime(7, true, true).is_err());
    assert!(bridge.set_runtime(6, true, true).is_err());
    assert!(bridge.admit(binding, &sender, &request).is_err());
    bridge.set_runtime(8, true, true).unwrap();
    assert!(bridge.admit(binding, &sender, &request).is_err());
}
#[test]
fn body_claims_never_supply_frame_metadata_or_authority() {
    assert!(parse_request(r#"{"windowGeneration":1,"runtimeGeneration":7,"operation":"bootstrap","payload":null,"origin":"dsh-app://app","isMainFrame":true}"#).is_err());
    assert!(
        parse_request(
            r#"{"windowGeneration":1,"runtimeGeneration":7,"operation":"shell","payload":null}"#
        )
        .is_err()
    );
    assert!(parse_request(r#"{"windowGeneration":1,"runtimeGeneration":7,"operation":"bootstrap","payload":{"cookie":"fixture"}}"#).is_err());
    assert!(parse_request(&"x".repeat(MAX_MESSAGE_BYTES + 1)).is_err());
    let (bridge, binding, sender, mut request) = fixture();
    request.window_generation += 1;
    assert!(bridge.admit(binding, &sender, &request).is_err());
}
