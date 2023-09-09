#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,Default)]
pub enum TerminationBytes{
    LF,
    CR,
    CRLF,
    #[default]
    Unkown,
    None,
    Custom(Vec<u8>)
}