use std::net::{SocketAddr, ToSocketAddrs};
///This function allows us to convert the hostname to an ip Address. This prioritizes an IPV4 address.
fn get_ipv4_first<T: AsRef<str>>(host_name: T) -> Option<SocketAddr> {
    let mut first_ipv6: Option<SocketAddr> = None;
    for addr in format!("{}:0", host_name.as_ref()).to_socket_addrs().ok()? {
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
