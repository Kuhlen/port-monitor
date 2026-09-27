use domain::AppError;
use domain::serial::BaudRate;

#[test]
fn parses_positive_baud() {
    let baud = BaudRate::parse(" 115200 ").expect("valid");
    assert_eq!(baud.get(), 115_200);
    assert_eq!(baud.to_string(), "115200");
}

#[test]
fn rejects_zero_and_junk() {
    for raw in ["0", "abc", "", "-9600"] {
        assert!(
            matches!(BaudRate::parse(raw), Err(AppError::Port(_))),
            "{raw:?}"
        );
    }
}
