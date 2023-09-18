use crate::connection::visa_conn;
use crate::{Error, InstConnection};
use hostname;
use lazy_static::lazy_static;
use regex::Regex;
use socket::*;
use std::borrow::Cow;
use std::ffi::CString;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::str::FromStr;
use visa_gpib::*;
use visa_socket::*;
use visa_vxi::*;

pub mod socket;
pub mod visa_gpib;
pub mod visa_socket;
pub mod visa_vxi;
// pub mod visa_hislip;
// pub mod visa_usb;

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum InstAddr {
    Visa(VisaAddress),
    Socket(Socket),
}

impl fmt::Display for InstAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.address())
    }
}

impl InstAddr {
    /// this function takes care of parsing and creating new instance for
    /// all types of instrument addresses if the address matches any of
    /// the regex patterns defined above it will assume that it will not
    /// match any other format. It will then attempt to parse it and return
    /// a Result. Note this function doesn't handle UTF8 host names yet.
    /// It is already being explored.
    /// ```rust
    /// use instrument_communication::address::InstAddr;
    /// use std::str::FromStr;
    /// let address:&str="TCPIP::192.168.0.1::INSTR";
    /// let method1= InstAddr::new(address).unwrap();
    /// let method2= InstAddr::from_str(address).unwrap();
    /// let method3=address.parse::<InstAddr>().unwrap();
    /// let method4:InstAddr=address.parse().unwrap();
    /// assert_eq!(method1,method2);
    /// assert_eq!(method2,method3);
    /// assert_eq!(method3,method4);
    /// ```
    pub fn new(address: impl AsRef<str>) -> Result<Self, String> {
        let address = address
            .as_ref()
            .split_whitespace()
            .collect::<String>()
            .to_ascii_lowercase();
        if let Some(captures) = GPIB_ADDRESS_REGEX.captures(&address) {
            parse_gpib(captures)
        } else if let Some(captures) = VISASOCKET_ADDRESS_REGEX.captures(&address) {
            parse_visa_socket(captures)
        } else if let Some(captures) = VISAVXI11_ADDRESS_REGEX.captures(&address) {
            parse_visa_vxi11(captures)
        } else {
            parse_socket(&address)
        }
    }

    pub fn address(&self) -> Cow<str> {
        match self {
            InstAddr::Visa(addr) => (&addr.address).into(),
            InstAddr::Socket(addr) => match addr {
                Socket::V4(socket) => format!("{}:{}", socket.ip(), socket.port()).into(),
                Socket::V6(socket) => format!("{}:{}", socket.ip(), socket.port()).into(),
                Socket::Raw(socket) => format!("{}:{}", socket.host_name, socket.port).into(),
            },
        }
    }
    ///Consume the address and return a communication interface
    pub fn connect(self) -> Result<Box<dyn InstConnection>, Error> {
        match self {
            InstAddr::Visa(addr) => addr.connect(),
            InstAddr::Socket(addr) => addr.connect(),
        }
    }
}
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub enum VisaType {
    GPIB,
    Socket,
    USB,
    Hislip,
    Serial,
    VXI,
}
impl FromStr for InstAddr {
    type Err = String;
    fn from_str(address: &str) -> Result<Self, Self::Err> {
        InstAddr::new(address)
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VisaAddress {
    address: String,
    visa_type: VisaType,
}
impl VisaAddress {
    fn connect(self) -> Result<Box<dyn InstConnection>, Error> {
        let connection = visa_conn::VisaConn::connect(self, None)?;
        Ok(Box::new(connection) as Box<dyn InstConnection>)
    }

    pub fn get_type(&self) -> VisaType {
        self.visa_type
    }
}
impl From<VisaAddress> for *const i8 {
    fn from(val: VisaAddress) -> Self {
        CString::new(val.address)
            .unwrap_or(CString::default())
            .as_ptr()
    }
}

impl From<VisaAddress> for InstAddr {
    fn from(val: VisaAddress) -> Self {
        InstAddr::Visa(val)
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct RawSocket {
    host_name: Cow<'static,str>,
    port: u16,
}

impl RawSocket {
    /// get a reference to the hostname 
    pub fn host_name<'a>(&'a self) -> Cow<'a, str> {
        (&self.host_name).clone()
    }
    /// Since RawSocket contains the hostname we don't have guarantee that the host name can be translated into an ip address.
    /// this function gets the SocketAddr object if it exists. The priority will be given to an ipv4 address but an ipv6 will
    /// be returned in the case where the hostname doesn't have an ipv4 address.
    pub fn get_ipv4_first(&self) -> Option<SocketAddr> {
        let mut first_ipv6: Option<SocketAddr> = None;
        for addr in format!("{}:0", &self.host_name).to_socket_addrs().ok()? {
            match addr {
                SocketAddr::V4(ipv4_addr) => return Some(SocketAddr::V4(ipv4_addr)),
                SocketAddr::V6(ipv6_addr) => {
                    if first_ipv6.is_none() {
                        first_ipv6 = Some(SocketAddr::V6(ipv6_addr));
                    }
                }
            }
        }
        first_ipv6
    }
}

impl std::fmt::Display for RawSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}",&self.host_name,&self.port))
    }
}
