use std::time::Duration;

use crate::{address::InstAddr, err::Error, termination_bytes::TerminationBytes};

pub trait InstConnection {
    fn address(&self) -> InstAddr;
    fn set_timeout(&self, timeout: Duration) -> Result<(), Error>;
    fn reconnect(&self) -> Result<(), Error>;
    fn set_termination(&mut self,term_bytes:TerminationBytes)-> Result<(), Error>;
}
