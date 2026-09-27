use domain::line_buffer::LineBuffer;

#[test]
fn splits_on_newline_and_trims_cr() {
    let mut buf = LineBuffer::default();
    assert_eq!(buf.push(b"a\r\nb\nc"), vec!["a", "b"]);
    assert_eq!(buf.flush(), Some("c".to_string()));
    assert_eq!(buf.flush(), None);
}

#[test]
fn empty_lines_dropped() {
    let mut buf = LineBuffer::default();
    assert_eq!(buf.push(b"\n\r\n\r\r\nx\n"), vec!["x"]);
}

#[test]
fn line_waits_for_newline() {
    let mut buf = LineBuffer::default();
    assert!(buf.push(b"par").is_empty());
    assert_eq!(buf.push(b"tial\n"), vec!["partial"]);
}

// old reader decoded per chunk: a split char became two U+FFFD
#[test]
fn multibyte_split_across_chunks_decodes_whole() {
    let mut buf = LineBuffer::default();
    assert!(buf.push(&[0xC3]).is_empty());
    assert_eq!(buf.push(&[0xA9, b'\n']), vec!["é"]);
}

#[test]
fn invalid_utf8_is_lossy() {
    let mut buf = LineBuffer::default();
    assert_eq!(buf.push(&[0xFF, b'\n']), vec!["\u{FFFD}"]);
}
