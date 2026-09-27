use domain::send::{LineEnding, encode};

#[test]
fn every_ending_appends_its_bytes() {
    for (ending, want) in [
        (LineEnding::None, &b"AT"[..]),
        (LineEnding::Lf, &b"AT\n"[..]),
        (LineEnding::Cr, &b"AT\r"[..]),
        (LineEnding::CrLf, &b"AT\r\n"[..]),
    ] {
        assert_eq!(encode("AT", ending), want, "{ending:?}");
    }
}

#[test]
fn crlf_is_default() {
    assert_eq!(LineEnding::default(), LineEnding::CrLf);
}
