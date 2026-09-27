#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    None,
    Lf,
    Cr,
    #[default]
    CrLf,
}

impl LineEnding {
    // order = send_bar.slint combo
    pub const ALL: [Self; 4] = [Self::None, Self::Lf, Self::Cr, Self::CrLf];

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::None => b"",
            Self::Lf => b"\n",
            Self::Cr => b"\r",
            Self::CrLf => b"\r\n",
        }
    }
}

pub fn encode(text: &str, ending: LineEnding) -> Vec<u8> {
    let mut out = text.as_bytes().to_vec();
    out.extend_from_slice(ending.bytes());
    out
}
