use port_monitor_core::AppError;

// Frontend decodes AppError coming from the backend. If a variant stops
// round-tripping, errors silently degrade into a raw string.
#[test]
fn every_variant_round_trips() {
    for e in [
        AppError::Port("x".into()),
        AppError::Validation("x".into()),
        AppError::Update("x".into()),
        AppError::AlreadyConnected,
        AppError::NotConnected,
        AppError::IpcUnavailable,
    ] {
        let json = serde_json::to_string(&e).unwrap();
        let back: AppError = serde_json::from_str(&json).unwrap();
        assert_eq!(e, back);
    }
}
