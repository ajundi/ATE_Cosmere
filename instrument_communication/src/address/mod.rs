use crate::{Error, InstConnection};
use hostname;
use lazy_static::lazy_static;
use regex::Regex;
use socket::*;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::vec;
use visa_gpib::*;
use visa_socket::*;
use visa_vxi11::*;

pub mod socket;
pub mod visa_gpib;
pub mod visa_socket;
pub mod visa_vxi11;
// pub mod visa_usb;
// pub mod visa_hislip;

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum InstAddr {
    VisaGPIB(VisaAddress<GPIB>),
    VisaVXI11(VisaAddress<VXI11>),
    VisaSocket(VisaAddress<Socket>),
    VisaHislip(VisaAddress<Hislip>),
    VisaUSB(VisaAddress<USB>),
    VisaSerial(VisaAddress<Serial>),
    VisaVXI(VisaAddress<VXI>),
    Socket { socket: SocketAddr, address: String },
}

impl InstAddr {
    pub fn address(&self) -> &str {
        match self {
            InstAddr::VisaGPIB(add) => &add.address,
            InstAddr::VisaVXI11(add) => &add.address,
            InstAddr::VisaSocket(add) => &add.address,
            InstAddr::VisaUSB(add) => &add.address,
            InstAddr::VisaSerial(add) => &add.address,
            InstAddr::VisaHislip(add) => &add.address,
            InstAddr::VisaVXI(add) => &add.address,
            InstAddr::Socket { socket: _, address } => address,
        }
    }
    pub fn open_connection(&self) -> Result<Box<dyn InstConnection>, Error> {
        todo!();
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct Socket;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VXI11;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct GPIB;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct USB;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct Serial;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct Hislip;
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VXI;

impl FromStr for InstAddr {
    type Err = String;
    ///this function takes care of parsing all types of instrument addresses
    /// if the address matches any of the regex patterns defined above it
    /// will assume that it will not match any other format. It will then
    /// attempt to parse it and return a Result.
    /// Note this function doesn't handle UTF8 host names yet. It is already
    /// being explored.
    /// ```rust
    /// use instrument_communication::address::InstAddr;
    /// use std::str::FromStr;
    /// let address:&str="TCPIP::192.168.0.1::INSTR";
    /// let method1= InstAddr::from_str(address).unwrap();
    /// let method2=address.parse::<InstAddr>().unwrap();
    /// let method3:InstAddr=address.parse().unwrap();
    /// assert_eq!(method1,method2);
    /// assert_eq!(method2,method3);
    /// ```
    fn from_str(address: &str) -> Result<Self, Self::Err> {
        let address = address
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
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VisaAddress<T> {
    address: String,
    visa_type: PhantomData<T>,
}
