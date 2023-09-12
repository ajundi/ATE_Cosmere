use crate::address::*;
use std::fmt::Display;
use std::net::Ipv6Addr;

const NUMBER_IPV4_OCTETS: usize = 4;

lazy_static! {
    pub static ref HOSTNAME_REGEX: Regex = Regex::new(
        r"^(?i)(([a-z0-9]|[a-z0-9][a-z0-9\-]*[a-z0-9])\.)*([a-z0-9]|[a-z0-9][a-z0-9\-]*[a-z0-9])$"
    )
    .unwrap();
}

pub fn parse_socket<T: AsRef<str>>(address: T) -> Result<InstAddr, String> {
    let socket: Socket = address.as_ref().parse()?;
    Ok(InstAddr::Socket(socket))
}

#[derive(Clone, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub enum Socket {
    V4(SocketAddrV4),
    V6(SocketAddrV6),
    Raw(RawSocket),
}

impl Socket {
    /// this function takes care of parsing socket addresses
    /// if the address matches any of the regex patterns defined above it
    /// will assume that it will not match any other format. It will then
    /// attempt to parse it and return a Result.
    /// Note this function doesn't handle UTF8 host names yet. It is already
    /// being explored. This is a non-standard parse IP implementation as It accepts IPs written
    /// with leading zeros such as 127.00.000.001  
    ///
    /// # Examples
    ///
    /// ```rust
    /// use instrument_communication::address::socket::Socket;
    /// use std::str::FromStr;
    /// let address:&str="192.168.0.001:5025";
    /// let method1= Socket::new(address).unwrap();
    /// let method2= Socket::from_str(address).unwrap();
    /// let method3=address.parse::<Socket>().unwrap();
    /// let method4:Socket=address.parse().unwrap();
    /// assert_eq!(method1,method2);
    /// assert_eq!(method2,method3);
    /// assert_eq!(method3,method4);
    /// ```
    pub fn new(address: impl AsRef<str>) -> Result<Self, String> {
        let splits: Vec<&str> = address.as_ref().split(':').map(str::trim).collect();
        if splits.len() < 2 {
            return Err("Incorrect Socket address format. Address format is shown inside the quotes \"IP:PortNumber\" Or \"HostName:PortNumber\" examples:\n 192.168.0.20:8080 or PCNAME1:50050".into());
        }
        let (port, ip) = splits
            .split_last()
            .expect("We already checked it has at least 2 elements");
        let port = port
            .parse::<u16>()
            .map_err(|_| format!("Unable to parse port into a number. port: {}", port))?;
        let ip = ip.join(":"); //if it is IPV4 there will be one &str and no change. If it is IPV6 they will be joined correctly.
        let ip = NetworkAddr::from_str(&ip)?;
        match ip {
            NetworkAddr::V4(addr) => Ok(Socket::V4(SocketAddrV4::new(addr, port))),
            NetworkAddr::V6(addr) => Ok(Socket::V6(SocketAddrV6::new(addr, port, 0, 0))),
            NetworkAddr::RAW(addr) => Ok(Socket::Raw(RawSocket {
                host_name: addr.to_owned(),
                port,
            })),
        }
    }

    pub fn ip_or_host(&self) -> Cow<str> {
        match self {
            Socket::V4(ref addr) => addr.ip().to_string().into(),
            Socket::V6(ref addr) => addr.ip().to_string().into(),
            Socket::Raw(addr) => (&addr.host_name).into(),
        }
    }

    pub const fn port(&self) -> u16 {
        match self {
            Socket::V4(addr) => addr.port(),
            Socket::V6(addr) => addr.port(),
            Socket::Raw(addr) => addr.port,
        }
    }

    pub fn connect(self) -> Result<Box<dyn InstConnection>, Error> {
        match self {
            Socket::V4(addr) => todo!(),
            Socket::V6(addr) => todo!(),
            Socket::Raw(addr) => todo!(),
        }
    }
}

impl FromStr for Socket {
    type Err = String;
    fn from_str(address: &str) -> Result<Self, Self::Err> {
        Socket::new(address)
    }
}

const LOCALHOST: &str = "localhost";
lazy_static! {
    pub static ref LOCAL_MACHINE: String = hostname::get()
        .unwrap_or_else(|_| LOCALHOST.into())
        .to_str()
        .unwrap()
        .to_owned();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum NetworkAddr {
    /// An IPv4 address.
    V4(Ipv4Addr),
    /// An IPv6 address.
    V6(Ipv6Addr),
    /// Raw host/computer name
    RAW(String),
}

impl FromStr for NetworkAddr {
    type Err = String;
    /// This is a non-standard IP and hostname implementation as It accepts IPs written
    /// with leading zeros such as 127.00.000.001  
    ///
    /// # Examples
    ///
    /// ```
    /// use instrument_communication::address::socket::parse_ip;
    ///
    /// assert_eq!(parse_ip("127.00.000.001"),parse_ip("127.0.0.1"));
    /// assert_eq!(parse_ip("127.00.000.001"),parse_ip("localhost "));
    /// assert_ne!(parse_ip("127.0.0.2"),parse_ip("127.0.0.1"));
    /// ```
    fn from_str(addr: &str) -> Result<NetworkAddr, String> {
        let ip_or_host: &str = addr.trim();
        let split_ip = ip_or_host.split('.').collect::<Vec<_>>();
        let count = split_ip.len();
        let is_ipv4 =
            (count, split_ip.clone().into_iter().all(is_u8)) == (NUMBER_IPV4_OCTETS, true);
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
            let address =
                IpAddr::from_str(&ip).map_err(|_| format!("Unable to parse IP address: {}", ip))?;
            match address {
                IpAddr::V4(ip) => Ok(NetworkAddr::V4(ip)),
                IpAddr::V6(ip) => Ok(NetworkAddr::V6(ip)),
            }
        } else if ip_or_host.eq_ignore_ascii_case(LOCALHOST)
            || ip_or_host.eq_ignore_ascii_case("::1")
            || ip_or_host.eq_ignore_ascii_case(&LOCAL_MACHINE)
        {
            Ok(NetworkAddr::V4(Ipv4Addr::LOCALHOST))
        } else if let Ok(ipv6) = ip_or_host.parse::<Ipv6Addr>() {
            Ok(NetworkAddr::V6(ipv6))
        } else if HOSTNAME_REGEX.is_match(ip_or_host) {
            Ok(NetworkAddr::RAW(ip_or_host.into()))
        } else {
            return Err(format!("Unable to parse IP or hostname: {}", ip_or_host));
        }
    }
}

impl Display for NetworkAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkAddr::V4(addr) => addr.fmt(f),
            NetworkAddr::V6(addr) => addr.fmt(f),
            NetworkAddr::RAW(addr) => f.write_str(addr),
        }
    }
}

/// checks if the string is a number between 0 and 255
/// ```rust
/// use instrument_communication::address::socket::is_u8;
/// assert!(is_u8("0"));
/// assert!(is_u8("11"));
/// assert!(is_u8("123"));
/// assert!(is_u8("254"));
/// assert!(is_u8("255"));
/// assert!(!is_u8("256"));
/// ```
fn is_u8(s: &str) -> bool {
    s.parse::<u8>().is_ok()
}

#[cfg(test)]
mod test {
    use crate::address::socket::*;
    use std::str::FromStr;
    #[test]
    fn testing_socket_address() {
        let address: &str = "192.168.0.1:5025";
        let method1 = Socket::from_str(address).unwrap();
        let method2 = address.parse::<Socket>().unwrap();
        let method3: Socket = address.parse().unwrap();
        assert_eq!(method1, method2);
        assert_eq!(method2, method3);
    }

    #[test]
    fn testing_localhost_and_its_ip_match() {
        assert_eq!(
            NetworkAddr::from_str("127.00.000.001"),
            NetworkAddr::from_str("127.0.0.1")
        );
        assert_eq!(
            NetworkAddr::from_str("127.00.000.001"),
            NetworkAddr::from_str("localhost ")
        );
        assert_ne!(
            NetworkAddr::from_str("127.0.0.2"),
            NetworkAddr::from_str("127.0.0.1")
        );
    }
}
