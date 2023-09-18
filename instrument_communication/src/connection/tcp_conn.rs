use crate::address::InstAddr;
use crate::connect;
use crate::termination_bytes::{self, TerminationBytes};
use crate::{address::socket::Socket, communication::InstConnection, err::Error};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
///This function allows us to convert the hostname to an ip Address. This prioritizes an IPV4 address.

pub static connect_timeout: Duration = Duration::from_secs(2);
const DEFAULT_BUFFER_SIZE: usize = 4096;
pub struct TcpConn {
    connection: TcpStream,
    address: Socket,
    buffer_size: usize,
    term_string: Option<TerminationBytes>,
    frame_size: Option<usize>,
    timeout: Duration,
}

impl TcpConn {
    pub fn connect(addr: Socket) -> Result<Box<dyn InstConnection>, Error> {
        let connection = get_tcp_stream(addr.clone())?;
        Ok(Box::new(TcpConn {
            connection,
            address: addr,
            buffer_size: DEFAULT_BUFFER_SIZE,
            term_string: Some(TerminationBytes::default()),
            frame_size: None,
            timeout: connect_timeout,
        }))
    }
}

fn get_tcp_stream(addr: Socket) -> Result<TcpStream, Error> {
    Ok(match addr {
        Socket::V4(addr) => TcpStream::connect_timeout(&SocketAddr::V4(addr), connect_timeout)
            .map_err(|e| {
                Error::ConnectionFailed(format!("Failed to connect. Error message:{:?}", e).into())
            })?,
        Socket::V6(addr) => TcpStream::connect_timeout(&SocketAddr::V6(addr), connect_timeout)
            .map_err(|e| {
                Error::ConnectionFailed(format!("Failed to connect. Error message:{:?}", e).into())
            })?,
        Socket::Raw(addr) => {
            if let Some(addr) = addr.get_ipv4_first() {
                TcpStream::connect_timeout(&addr, connect_timeout).map_err(|e| {
                    Error::ConnectionFailed(
                        format!("Failed to connect. Error message:{:?}", e).into(),
                    )
                })?
            } else {
                Err(Error::ConnectionFailed(
                    format!("Unable to connect to hostname: {addr}").into(),
                ))?
            }
        }
    })
}

impl InstConnection for TcpConn {
    fn address(&self) -> crate::address::InstAddr {
        InstAddr::Socket(self.address.clone())
    }

    fn set_timeout(&self, timeout: Duration) -> Result<(), Error> {
        self.connection
            .set_read_timeout(Some(timeout))
            .map_err(|e| {
                Error::FunctionFailure(
                    format!("Failed to set connection timeout. Error: {e}").into(),
                )
            })?;
        self.connection
            .set_write_timeout(Some(timeout))
            .map_err(|e| {
                Error::FunctionFailure(
                    format!("Failed to set connection timeout. Error: {e}").into(),
                )
            })?;
        Ok(())
    }

    fn reconnect(&self) -> Result<(), Error> {
        todo!()
    }

    fn set_termination(&mut self, term_bytes: TerminationBytes) -> Result<(), Error> {
        match term_bytes {
            TerminationBytes::None=> match self.frame_size  {
                None=>Err(Error::ConflictingSettings(format!("Cannot set no termination when frame size is not fixed. We will not know when to return. Typically this is not a problem but some instruments might send data in bursts and an early return will cause a problem.").into()))?,
                _=>self.term_string=Some(term_bytes),
            }
            _=> self.term_string=Some(term_bytes),
        }
        Ok(())
    }
}
