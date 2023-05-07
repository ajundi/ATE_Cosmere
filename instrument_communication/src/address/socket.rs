use crate::address::*;

const NUMBER_IPV4_OCTETS: usize = 4;

pub fn parse_socket<T: AsRef<str>>(address: T) -> Result<InstAddr, String> {
    let socket: SocketAddr = socket_parsing(address)?;
    Ok(InstAddr::Socket {
        socket,
        address: socket.to_string(),
    })
}

pub fn socket_parsing<T: AsRef<str>>(address: T) -> Result<SocketAddr, String> {
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

pub fn resolve_ip(ip: &str) -> Result<IpAddr, String> {
    let split_ip = ip.split('.').collect::<Vec<_>>();
    let count = split_ip.len();
    let is_ipv4 = (count, split_ip.clone().into_iter().all(is_u8)) == (NUMBER_IPV4_OCTETS, true);
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
    let mut first_ipv6: Option<SocketAddr> = None;
    for addr in adds {
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

///checks if the string is a number between 0 and 255
/// ```rust 
/// use instrument_communication::address::socket::is_u8;
/// assert!(is_u8("0"));
/// assert!(is_u8("11"));
/// assert!(is_u8("123"));
/// assert!(is_u8("254"));
/// assert!(is_u8("255"));
/// assert!(!is_u8("256"));
/// ```
pub fn is_u8(s: &str) -> bool {
    match s.parse::<u8>() {
        Ok(_) => true,
        Err(_) => false,
    }
}
