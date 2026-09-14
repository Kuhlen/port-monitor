use port_monitor_core::features::filter::LineFilter;

#[test]
fn disabled_passes_through() {
    let f = LineFilter::default();
    assert_eq!(f.apply("ABC123"), Some("ABC123".into()));
}

#[test]
fn offset_and_length_slice() {
    let f = LineFilter {
        enabled: true,
        offset: 2,
        length: Some(3),
        exclude: String::new(),
    };
    assert_eq!(f.apply("ABCDEFG"), Some("CDE".into()));
}

#[test]
fn offset_past_end_drops_line() {
    let f = LineFilter {
        enabled: true,
        offset: 10,
        ..Default::default()
    };
    assert_eq!(f.apply("ABC"), None);
}

#[test]
fn exclude_strips_chars() {
    let f = LineFilter {
        enabled: true,
        exclude: "-:".into(),
        ..Default::default()
    };
    assert_eq!(f.apply("12-34:56"), Some("123456".into()));
}

// Byte slicing used to panic here.
#[test]
fn multibyte_does_not_panic() {
    let f = LineFilter {
        enabled: true,
        offset: 1,
        length: Some(2),
        ..Default::default()
    };
    assert_eq!(f.apply("aé€b"), Some("é€".into()));
}
