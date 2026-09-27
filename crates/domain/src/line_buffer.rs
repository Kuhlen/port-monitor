// bytes, not String: a UTF-8 char split across reads must decode whole
#[derive(Debug, Default)]
pub struct LineBuffer {
    pending: Vec<u8>,
}

impl LineBuffer {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        self.pending.extend_from_slice(bytes);
        let mut lines = Vec::new();
        while let Some(pos) = self.pending.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=pos).collect();
            lines.extend(decode(&line[..pos]));
        }
        lines
    }

    /// newline-less tail, called on read timeout
    pub fn flush(&mut self) -> Option<String> {
        decode(&std::mem::take(&mut self.pending))
    }
}

// old reader: trim_end_matches('\r'), empty lines dropped
fn decode(line: &[u8]) -> Option<String> {
    let end = line.iter().rposition(|&b| b != b'\r').map_or(0, |i| i + 1);
    let line = &line[..end];
    (!line.is_empty()).then(|| String::from_utf8_lossy(line).into_owned())
}
