use std::borrow::Cow;

#[non_exhaustive]
pub enum Error<'a> {
    Timeout,
    ParseFailed(Cow<'a, str>),
}
