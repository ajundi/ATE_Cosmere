#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TerminationBytes {
    #[default]
    LF,
    CR,
    CRLF,
    None,
    Custom(Vec<u8>),
}

impl TerminationBytes {
    pub fn bytes(&self) -> &[u8] {
        match self {
            TerminationBytes::LF => &[10],
            TerminationBytes::CR => &[13],
            TerminationBytes::CRLF => &[13, 10],
            TerminationBytes::None => &[],
            TerminationBytes::Custom(bytes) => bytes.as_slice(),
        }
    }
}
