use std::borrow::Cow;
#[non_exhaustive]
#[derive(Debug,Clone)]
pub enum Error {
    Timeout,
    BinaryError(Cow<'static, str>),
    OpenSessionError(Cow<'static, str>),
    ParseFailed(Cow<'static, str>),
    ConnectionFailed(Cow<'static, str>),
}
