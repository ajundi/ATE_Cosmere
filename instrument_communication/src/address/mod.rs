use crate::{Error, InstConnection};
use hostname;
use lazy_static::lazy_static;
use regex::Regex;
use socket::*;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4,SocketAddrV6};
use std::str::FromStr;
use std::fmt;
use visa_gpib::*;
use visa_socket::*;
use visa_vxi::*;
use std::borrow::Cow;

pub mod socket;
pub mod visa_gpib;
pub mod visa_vxi;
pub mod visa_socket;
// pub mod visa_hislip;
// pub mod visa_usb;

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum InstAddr {
    VisaGPIB(VisaAddress<GPIB>),
    VisaVXI11(VisaAddress<VXI>),
    VisaSocket(VisaAddress<Socket>),
    VisaHislip(VisaAddress<Hislip>),
    VisaUSB(VisaAddress<USB>),
    VisaSerial(VisaAddress<Serial>),
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
    pub fn new(address:impl AsRef<str>)->Result<Self, String>{
        let address = address.as_ref()
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
            InstAddr::VisaGPIB(addr) => (&addr.address).into(),
            InstAddr::VisaVXI11(addr) => (&addr.address).into(),
            InstAddr::VisaSocket(addr) => (&addr.address).into(),
            InstAddr::VisaUSB(addr) => (&addr.address).into(),
            InstAddr::VisaSerial(addr) => (&addr.address).into(),
            InstAddr::VisaHislip(addr) => (&addr.address).into(),
            InstAddr::Socket(addr)=> match addr {
                Socket::V4(socket) => format!("{}:{}", socket.ip(), socket.port()).into(),
                Socket::V6(socket) => format!("{}:{}", socket.ip(), socket.port()).into(),
                Socket::Raw(socket) => format!("{}:{}", socket.host_name, socket.port).into(),
            }
        }
    }
    ///Consume the address and return a communication interface
    pub fn connect(self) -> Result<Box<dyn InstConnection>, Error<'static>> {
        todo!();
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VXI;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct GPIB;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct USB;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct Serial;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct Hislip;

impl FromStr for InstAddr {
    type Err = String;
    fn from_str(address: &str) -> Result<Self, Self::Err> {
        InstAddr::new(address)
    }
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VisaAddress<T> {
    address: String,
    visa_type: PhantomData<T>,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct RawSocket{
    host_name: String ,
    port: u16,
}