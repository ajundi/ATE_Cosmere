use crate::err::Error;


pub trait InstConnection {
    fn address(&self) -> &dyn AsRef<str>;
    fn set_timeout(&self, timeout: u64) -> Result<(), Error>;
    fn reconnect(&self) -> Result<(), Error>;
}