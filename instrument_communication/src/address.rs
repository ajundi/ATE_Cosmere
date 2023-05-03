use lazy_static::lazy_static;
use regex::Regex;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use crate::{InstConnection, Error};

pub trait InstAddress {
    fn address(&self) -> String;
    fn open_connection(&self) -> Result<Box<dyn InstConnection>, Error>;
}

#[derive(Debug)]
pub struct TCPIP;
#[derive(Debug)]
pub struct GPIB;
#[derive(Debug)]
pub struct USB;
#[derive(Debug)]
pub struct Serial;
#[derive(Debug)]
pub struct Hislip;
#[derive(Debug)]
pub struct VXI;


pub fn create(address: &dyn AsRef<str>) -> Option<Box<dyn InstAddress>> {
    visa_parse(address.as_ref())
    .or_else(|| tcp_parse(address.as_ref()).ok())
}

const NUMBER_IP_INTS: usize = 4;

fn tcp_parse(address: &str) -> Result<Box<dyn InstAddress>, String> {
    let splits: Vec<&str> = address.split(":").map(str::trim).collect();
    if splits.len() < 2 {
        return Err(format!("Incorrect Socket address format. Address format is shown inside the quotes \"IP:PortNumber\" Or \"HostName:PortNumber\" examples:\n 192.168.0.20:8080 or PCNAME1:50050"));
    }
    let (port, ip) = splits
        .split_last()
        .expect("We already checked it has at least 2 elements");
    let port = port
        .parse::<u16>()
        .map_err(|_| format!("Unable to parse port into a number. port: {}", port))?;
    let ip = ip.join(":");
    let ip_or_host = resolve_ip(&ip)?;
    Ok(Box::new(SocketAddr::new(ip_or_host, port)))
}

fn resolve_ip(ip: &str) -> Result<IpAddr, String> {
    let split_ip = &mut ip.split('.');
    let count = ip.split('.').count();
    let is_ipv4 = match (count, split_ip.all(is_u8)) {
        (NUMBER_IP_INTS, true) => true,
        _ => false,
    };
    if is_ipv4 {
        let ip = split_ip
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
    } else if ip.eq_ignore_ascii_case("localhost") || ip.eq_ignore_ascii_case("::1") {
        return Ok(IpAddr::V4(Ipv4Addr::LOCALHOST));
    } else if let Ok(ipv4) = ip.parse::<std::net::Ipv6Addr>() {
        return Ok(IpAddr::V6(ipv4));
    } else {
        if let Ok(addrs) = &mut (ip.to_owned() + ":0").to_socket_addrs() {
            if let Some(addr) = addrs.next() {
                return Ok(addr.ip());
            }
        }
    }
    Err(format!("Unable to resolve IP address: {}", ip))
}

fn is_u8(s: &str) -> bool {
    match s.parse::<u8>() {
        Ok(_) => true,
        Err(_) => false,
    }
}
#[derive(Debug, Clone)]
pub struct VisaAddress<T> {
    address: String,
    visa_type: PhantomData<T>,
}

lazy_static! {
    pub static ref GPIB_ADDRESS_REGEX: Regex =
        Regex::new(r"^(?i)(GPIB\d+)::(\d+)(?:::\d+)?::INSTR$").unwrap();
}

fn visa_parse(address: &str) -> Option<Box<dyn InstAddress>> {
    let address = address.split_whitespace().collect::<String>();
    if let Some(captures) = GPIB_ADDRESS_REGEX.captures(&address) {
        let instr_num = captures[2].to_string();
        if  instr_num.len()<3
          &&instr_num.cmp(&"31".to_owned())==std::cmp::Ordering::Less {
            let address = format!("{}::{}::instr", captures[1].to_ascii_lowercase(), instr_num);
            return Some(Box::new(VisaAddress {
                address,
                visa_type: PhantomData::<GPIB>,
                }));
            }
    }
    None
}

impl<T> InstAddress for VisaAddress<T> {
    fn address(&self) -> String {
        self.address.to_owned()
    }

    fn open_connection(&self) -> Result<Box<dyn InstConnection>, Error> {
        todo!()
    }
}

impl VisaAddress<GPIB> {
    #[allow(dead_code)]
    fn complement_address(&self) -> String {
        let address_parts: Vec<&str> = self.address.split("::").collect();
        let mut num = address_parts[1].parse::<i32>().unwrap();
        num += (num % 2 == 0).then(|| 1).unwrap_or(-1);
        return format!(
            "{}::{}::{}",
            address_parts[0],
            num,
            address_parts[2]
        );
    }
}

impl InstAddress for SocketAddr {
    fn address(&self) -> String {
        self.to_string()
    }
    
    fn open_connection(&self) -> Result<Box<dyn InstConnection>, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("GPIB0::15::INSTR";"This is basic format for a GPIB address.")]
    #[test_case("GPIB3::1::INSTR";"GPIB address board number 3 GPIB address instrument number 1.")]
    #[test_case("GPIB0::1::12::INSTR";"Address format with primary and secondary address. Only primary will be used.")]
    #[test_case("GPIB1::2::instr";"Case shouldn't matter.")]
    #[test_case("gpib1::20::instr";"Why not another one with all of them small letters.")]
    fn test_gpib_address_regex_with_valid_addresses(address: &str) {
        assert!(GPIB_ADDRESS_REGEX.is_match(address));
    }
    #[test_case("GPIB0::15::INSTRx";"having additional characters after INSTR is not valid.")]
    #[test_case("";"blank address is not valid.")]
    #[test_case("GPIB::15::INSTR";"no GPIB board number provided.")]
    #[test_case("GPIB2 :: 40::12:: INSTR ";"addresses above 30 are not valid.")]
    #[test_case("GPIB0:16::INSTR";"Missing colon between board number and instrument number.")]
    #[test_case("GPIB0::16:INSTR";"Missing colon between instrument number and INSTR.")]
    #[test_case("GPIB0::16::instr1";"number after instr.")]
    #[test_case("GPIB0::16";"missing colons and instr.")]
    
    fn test_gpib_parse_address_with_incorrect_addresses(address: &str) {
        assert!(!GPIB_ADDRESS_REGEX.is_match(address));
    }

    #[test_case("GPIB0::16::INSTR", "GPIB0::17::INSTR")]
    #[test_case("GPIB0::15::INSTR", "GPIB0::14::INSTR")]
    #[test_case("GPIB1::22::INSTR", "GPIB1::23::INSTR")]
    fn test_complement_address(original: &str, expected: &str) {
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
    fn test_complement_address_fails(original: &str, expected: &str) {
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
fn test_visa_parse_valid_address(address:&str, expected:&str) {
    let inst_address = visa_parse(address).unwrap();
    assert!(inst_address.address().eq_ignore_ascii_case(expected));
}

#[test_case("GPIB0::15::INSTRx";"having additional characters after INSTR is not valid.")]
#[test_case("";"blank address is not valid.")]
#[test_case("GPIB::15::INSTR";"no GPIB board number provided.")]
#[test_case("GPIB2 :: 40::12:: INSTR ";"addresses above 30 are not valid.")]
#[test_case("GPIB2 :: 220::12:: INSTR ";"addresses above 30 are not valid. Here is an example with a number that starts with 2 locations that are less than 30.")]
fn test_visa_parse_invalid_address(address:&str) {
    let inst_address = visa_parse(address);
    assert!(inst_address.is_none());
}

}
