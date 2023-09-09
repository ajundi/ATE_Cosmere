use std::time::Duration;

use crate::{err::Error, address::InstAddr};

pub trait InstConnection {
    fn address(&self) -> InstAddr;
    fn set_timeout(&self, timeout: Duration) -> Result<(), Error>;
    fn reconnect(&self) -> Result<(), Error>;
}
