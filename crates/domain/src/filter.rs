// Console line filter. Pure: tested without UI.

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LineFilter {
    pub enabled: bool,
    pub offset: usize,
    pub length: Option<usize>,
    pub exclude: String,
}

/// struck-through parts in the filter bar; exclude chars not shown
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterPreview {
    pub skipped: String,
    pub kept: String,
    pub cut: String,
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

        if out.is_empty() { None } else { Some(out) }
    }

    pub fn preview(&self, line: &str) -> FilterPreview {
        let skipped = line.chars().take(self.offset).collect();
        let rest = line.chars().skip(self.offset);
        let (kept, cut) = match self.length {
            Some(n) => (rest.clone().take(n).collect(), rest.skip(n).collect()),
            None => (rest.collect(), String::new()),
        };
        FilterPreview { skipped, kept, cut }
    }
}
