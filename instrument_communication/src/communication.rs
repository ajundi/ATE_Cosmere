use crate::{err::Error, address::InstAddr};

pub trait InstConnection {
    fn address(&self) -> InstAddr;
    fn set_timeout(&self, timeout: u64) -> Result<(), Error>;
    fn reconnect(&self) -> Result<(), Error>;
}
