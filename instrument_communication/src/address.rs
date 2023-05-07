use crate::{Error, InstConnection};
use hostname;
use lazy_static::lazy_static;
use regex::Regex;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::vec;

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

lazy_static! {
 pub static ref GPIB_ADDRESS_REGEX: Regex =
     Regex::new(r"^(?i)GPIB(\d*)::(\d+)(?:::\d+)?(?:::INSTR)?$").unwrap();
 pub static ref VISASOCKET_ADDRESS_REGEX: Regex =
     Regex::new(r"^(?i)TCPIP(\d*)::((?:[0-9]{1,3}\.){3}[0-9]{1,3}|(?:(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9])\.)*(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9]))::(\d+)::SOCKET$").unwrap();
 pub static ref VISAVXI11_ADDRESS_REGEX: Regex =
     Regex::new(r"^(?i)TCPIP(\d*)::((?:[0-9]{1,3}\.){3}[0-9]{1,3}|(?:(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9])\.)*(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9]))(?:::INSTR)?$").unwrap();
}
impl FromStr for InstAddr {
    type Err = String;
    ///this function takes care of parsing all types of instrument addresses
    /// if the address matches any of the regex patterns defined above it
    /// will assume that it will not match any other format. It will then
    /// attempt to parse it and return a Result.
    /// Note this function doesn't handle UTF8 host names yet. It is already
    /// being explored. Example
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
            tcp_parse(&address)
        }
    }
}

fn parse_gpib(captures: regex::Captures) -> Result<InstAddr, String> {
    let instr_num = captures[2].to_string();
    if instr_num.len() < 3 && instr_num.cmp(&"31".to_owned()) == std::cmp::Ordering::Less {
        let board_num = if captures[1].len() == 0 {
            "0".to_owned()
        } else {
            captures[1].to_string()
        };
        let address = format!("gpib{}::{}::instr", board_num, instr_num);
        return Ok(InstAddr::VisaGPIB(VisaAddress {
            address,
            visa_type: PhantomData::<GPIB>,
        }));
    } else {
        Err(format!("Invalid primary GPIB address {}", instr_num))
    }
}

fn parse_visa_socket(captures: regex::Captures) -> Result<InstAddr, String> {
    let board_num = captures[1].to_string();
    let board_num = if board_num.len() == 0 {
        "0".to_owned()
    } else {
        board_num
    };
    let host_ip = captures[2].to_string();
    let socket: SocketAddr = socket_parsing(&format!("{}:{}", host_ip, captures[3].to_string()))?;
    return Ok(InstAddr::VisaSocket(VisaAddress {
        address: format!(
            "tcpip{}::{}::{}::socket",
            board_num,
            socket.ip(),
            socket.port()
        ),
        visa_type: PhantomData::<Socket>,
    }));
}

fn parse_visa_vxi11(captures: regex::Captures) -> Result<InstAddr, String> {
    let board_num = captures[1].to_string();
    let board_num = if board_num.len() == 0 {
        "0".to_owned()
    } else {
        board_num
    };
    let host_ip = captures[2].to_string();
    let ip_or_host = resolve_ip(&host_ip)?;
    return Ok(InstAddr::VisaVXI11(VisaAddress {
        address: format!("tcpip{}::{}::instr", board_num, ip_or_host),
        visa_type: PhantomData::<VXI11>,
    }));
}

const NUMBER_IP_INTS: usize = 4;

fn tcp_parse<T: AsRef<str>>(address: T) -> Result<InstAddr, String> {
    let socket: SocketAddr = socket_parsing(address)?;
    Ok(InstAddr::Socket {
        socket: socket,
        address: socket.to_string(),
    })
}

fn socket_parsing<T: AsRef<str>>(address: T) -> Result<SocketAddr, String> {
    let splits: Vec<&str> = address.as_ref().split(":").map(str::trim).collect();
    if splits.len() < 2 {
        return Err(format!("Incorrect Socket address format. Address format is shown inside the quotes \"IP:PortNumber\" Or \"HostName:PortNumber\" examples:\n 192.168.0.20:8080 or PCNAME1:50050"));
    }
    let (port, ip) = splits
        .split_last()
        .expect("We already checked it has at least 2 elements");
    let port = port
        .parse::<u16>()
        .map_err(|_| format!("Unable to parse port into a number. port: {}", port))?;
    let ip = ip.join(":"); //if it is IPV4 there will be one &str and no change. If it is IPV6 they will be joined correctly.
    let ip_or_host = resolve_ip(&ip)?;
    let socket = SocketAddr::new(ip_or_host, port);
    Ok(socket)
}
lazy_static! {
    pub static ref HOSTNAME: String = hostname::get()
        .unwrap_or_else(|_| "localhost".into())
        .to_str()
        .unwrap()
        .to_owned();
}

fn resolve_ip(ip: &str) -> Result<IpAddr, String> {
    let split_ip = ip.split('.').collect::<Vec<_>>();
    let count = split_ip.len();
    let is_ipv4 = (count, split_ip.clone().into_iter().all(is_u8)) == (NUMBER_IP_INTS, true);
    if is_ipv4 {
        let ip = split_ip
            .into_iter()
            .map(|num| {
                let cleaned = num.trim_start_matches('0');
                if cleaned.is_empty() {
                    "0"
                } else {
                    cleaned
                }
            })
            .collect::<Vec<_>>()
            .join(".");
        return IpAddr::from_str(&ip).map_err(|_| format!("Unable to parse IP address: {}", ip));
    } else if ip.eq_ignore_ascii_case("localhost")
        || ip.eq_ignore_ascii_case("::1")
        || ip.eq_ignore_ascii_case(&HOSTNAME)
    {
        return Ok(IpAddr::V4(Ipv4Addr::LOCALHOST));
    } else if let Ok(ipv4) = ip.parse::<std::net::Ipv6Addr>() {
        return Ok(IpAddr::V6(ipv4));
    } else {
        if let Ok(addrs) = &mut (ip.to_owned() + ":0").to_socket_addrs() {
            if let Some(addr) = get_ipv4_first(addrs) {
                return Ok(addr.ip());
            }
        }
    }
    Err(format!("Unable to resolve IP address: {}", ip))
}
fn get_ipv4_first(adds: &mut vec::IntoIter<SocketAddr>) -> Option<SocketAddr> {
    match adds.next() {
        Some(SocketAddr::V4(ipv4_addr)) => Some(SocketAddr::V4(ipv4_addr)),
        Some(SocketAddr::V6(ipv6_addr)) => match adds.next() {
            Some(SocketAddr::V4(ipv4_addr)) => Some(SocketAddr::V4(ipv4_addr)),
            Some(SocketAddr::V6(ipv6_addr)) => Some(SocketAddr::V6(ipv6_addr)),
            None => Some(SocketAddr::V6(ipv6_addr)),
        },
        None => None,
    }
}
fn is_u8(s: &str) -> bool {
    match s.parse::<u8>() {
        Ok(_) => true,
        Err(_) => false,
    }
}
#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct VisaAddress<T> {
    address: String,
    visa_type: PhantomData<T>,
}

impl VisaAddress<GPIB> {
    #[allow(dead_code)]
    fn complement_address(&self) -> String {
        let address_parts: Vec<&str> = self.address.split("::").collect();
        let mut num = address_parts[1].parse::<i32>().unwrap();
        num += (num % 2 == 0).then(|| 1).unwrap_or(-1);
        return format!("{}::{}::{}", address_parts[0], num, address_parts[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    #[test_case("GPIB0::16::INSTR", "GPIB0::17::INSTR")]
    #[test_case("GPIB0::15::INSTR", "GPIB0::14::INSTR")]
    #[test_case("GPIB1::22::INSTR", "GPIB1::23::INSTR")]
    fn test_gpib_complement_address(original: &str, expected: &str) {
        let address = VisaAddress {
            address: String::from(original),
            visa_type: PhantomData::<GPIB>,
        };
        assert_eq!(&address.complement_address(), expected);
    }

    #[test_case("GPIB0::18::INSTR", "GPIB0::17::INSTR")]
    #[test_case("GPIB0::15::INSTR", "GPIB0::13::INSTR")]
    #[test_case("GPIB1::16::INSTR", "GPIB0::17::INSTR")]
    #[test_case("GPIB1::16::INSTR", "GPIB0::17::INST")]
    #[test_case("GPIB1::16::INSTR", "GPIB0::17::INSTD")]
    fn test_gpib_complement_address_fails(original: &str, expected: &str) {
        let address = VisaAddress {
            address: String::from(original),
            visa_type: PhantomData::<GPIB>,
        };
        assert!(!address.complement_address().eq_ignore_ascii_case(expected));
    }

    #[test_case("GPIB0::15::INSTR","gpib0::15::instr";"This is basic format for a GPIB address.")]
    #[test_case("GPIB0 :: 15:: INSTR ","gpib0::15::instr";"spaces between main part of the address and the instrument number are allowed in this libray.")]
    #[test_case("GPIB0: :15::INSTR","gpib0::15::instr";"Spaces between colons.")]
    #[test_case("GPIB0::1 5::INSTR","gpib0::15::instr";"Spaces between numbers.")]
    #[test_case("GPIB0::30::INSTR","gpib0::30::instr";"30 is the maximum number allowed for GPIB instrument.")]
    #[test_case("GPIB0 :: 1::12:: INSTR ","gpib0::1::instr";"only primary address is used. Secondary address is ignored.")]
    #[test_case("GPIB1::0::INSTR","gpib1::0::instr";"0 is the minimum number allowed for GPIB instrument.")]
    #[test_case("gpib2 :: 1::12:: insTR ","gpib2::1::instr";"tolerate character cases.")]
    #[test_case("GPIB::15::INSTR","GPIB0::15::INSTR";"no GPIB board number provided defaults to 0.")]
    #[test_case("GPIB::13","GPIB0::13::INSTR";"no GPIB board number provided defaults to 0 and no INSTR.")]
    fn test_gpib_parse_valid_address(address: &str, expected: &str) {
        let inst_address: InstAddr = address.parse().unwrap();
        assert!(inst_address.address().eq_ignore_ascii_case(expected));
    }

    #[test_case("GPIB0::15::INSTRx";"having additional characters after INSTR is not valid.")]
    #[test_case("";"blank address is not valid.")]
    #[test_case("GPIB2 :: 40::12:: INSTR ";"addresses above 30 are not valid.")]
    #[test_case("GPIB2 :: 220::12:: INSTR ";"addresses above 30 are not valid. Here is an example with a number that starts with 2 locations that are less than 30.")]
    fn test_gpib_parse_invalid_address(address: &str) {
        assert!(address.parse::<InstAddr>().is_err());
    }

    #[test_case("TCPIP0 :: 192.168.0.1::5025:: SockEt ","TCPIP0::192.168.0.1::5025::socket";"tolerate character cases for socket.")]
    #[test_case("TCPIP :: 192.168.0.1::5025:: SockEt ","TCPIP0::192.168.0.1::5025::socket";"tolerate missing board number")]
    fn test_visa_socket_valid_address(address: &str, expected: &str) {
        let inst_address = address.parse::<InstAddr>();
        assert!(inst_address.is_ok());
        assert!(inst_address
            .unwrap()
            .address()
            .eq_ignore_ascii_case(expected));
    }

    #[test_case("TCPIP0 :: 192.168.0.1:: insTR ","TCPIP0::192.168.0.1::instr";"tolerate character cases.")]
    #[test_case("TCPIP::192.168.0.1::INSTR ","tcpip0::192.168.0.1::instr";"tolerate missing board number.")]
    fn test_visa_vxi11_valid_address(address: &str, expected: &str) {
        let inst_address = address.parse::<InstAddr>();
        assert!(inst_address.is_ok());
        assert!(inst_address
            .unwrap()
            .address()
            .eq_ignore_ascii_case(expected));
    }

    #[test]
    fn test_machine_name_is_local_host() {
        let inst_address = format!("TCPIP::{}::INSTR", HOSTNAME.as_str()).parse::<InstAddr>();
        assert!(inst_address.is_ok());
        assert!(inst_address
            .unwrap()
            .address()
            .eq_ignore_ascii_case("tcpip0::127.0.0.1::INSTR"));
    }

    #[test]
    fn test_hostname_regex() {
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP1::google.com"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP2::amazon.com::INSTR"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP3::mail.example.com"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP4::ftp.test-site.com"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP5::test-site.com"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP6::localhost::INSTR"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP7::127.0.0.1"));
        assert!(
            VISAVXI11_ADDRESS_REGEX.is_match("TCPIP8::a-very-long-hostname-with-numbers-123.com")
        );
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP9::mail.example.co.uk"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP10::a-host-name-with-dashes.com"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP11::example.co.uk"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP12::127.0.0.1::INSTR"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP13::a.b.c.d"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP14::a.b-c.d"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP15::www.example.com"));
        assert!(VISAVXI11_ADDRESS_REGEX
            .is_match("TCPIP16::a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP17::test.example.co.uk"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP18::a.host-name.with.dots.com"));
        assert!(VISAVXI11_ADDRESS_REGEX.is_match("TCPIP19::a.b.c"));
    }
}
