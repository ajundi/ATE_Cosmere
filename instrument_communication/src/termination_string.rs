

pub enum TerminationString{
    LF,
    CR,
    CRLF,
    Unkown,
    None,
    Custom(Vec<u8>)
}
