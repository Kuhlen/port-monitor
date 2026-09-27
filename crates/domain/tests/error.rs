use domain::AppError;

// messages land in console rows; old wording kept
#[test]
fn messages_keep_old_wording() {
    assert_eq!(AppError::Port("x".into()).to_string(), "port: x");
    assert_eq!(AppError::AlreadyConnected.to_string(), "already connected");
    assert_eq!(AppError::NotConnected.to_string(), "not connected");
    assert_eq!(
        AppError::PermissionDenied("/dev/ttyS0".into()).to_string(),
        "permission denied: /dev/ttyS0"
    );
}
