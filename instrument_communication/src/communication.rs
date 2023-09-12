use std::time::Duration;

use crate::{address::InstAddr, err::Error};

pub trait InstConnection {
    fn address(&self) -> InstAddr;
    fn set_timeout(&self, timeout: Duration) -> Result<(), Error>;
    fn reconnect(&self) -> Result<(), Error>;
}
