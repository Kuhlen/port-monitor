// Console line filter. Pure, never crosses IPC - lives in core so it can be
// tested without wasm.

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LineFilter {
    pub enabled: bool,
    pub offset: usize,
    pub length: Option<usize>,
    pub exclude: String,
}

impl LineFilter {
    // None = drop the line.
    pub fn apply(&self, line: &str) -> Option<String> {
        if !self.enabled {
            return Some(line.to_string());
        }

        // Slice by char, not byte: from_utf8_lossy can emit multibyte and a
        // byte slice landing mid-char panics.
        let mut out: String = match self.length {
            Some(n) => line.chars().skip(self.offset).take(n).collect(),
            None => line.chars().skip(self.offset).collect(),
        };
        if out.is_empty() {
            return None;
        }

        for ch in self.exclude.chars() {
            out = out.replace(ch, "");
        }

        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }
}
