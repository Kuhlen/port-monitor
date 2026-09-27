use domain::filter::{FilterPreview, LineFilter};

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

fn preview(offset: usize, length: Option<usize>, line: &str) -> FilterPreview {
    LineFilter {
        enabled: true,
        offset,
        length,
        exclude: String::new(),
    }
    .preview(line)
}

fn parts(skipped: &str, kept: &str, cut: &str) -> FilterPreview {
    FilterPreview {
        skipped: skipped.into(),
        kept: kept.into(),
        cut: cut.into(),
    }
}

#[test]
fn preview_splits_skipped_kept_cut() {
    assert_eq!(preview(2, Some(3), "ABCDEFG"), parts("AB", "CDE", "FG"));
}

#[test]
fn preview_without_length_keeps_rest() {
    assert_eq!(preview(2, None, "ABCDEFG"), parts("AB", "CDEFG", ""));
}

#[test]
fn preview_offset_past_end_skips_all() {
    assert_eq!(preview(10, None, "ABC"), parts("ABC", "", ""));
}

#[test]
fn preview_counts_chars_not_bytes() {
    assert_eq!(preview(1, Some(2), "aé€b"), parts("a", "é€", "b"));
}
