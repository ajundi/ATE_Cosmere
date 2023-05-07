use super::socket::socket_parsing;
use crate::address::*;

lazy_static! {
    pub static ref VISASOCKET_ADDRESS_REGEX: Regex =
     Regex::new(r"^(?i)TCPIP(\d*)::((?:[0-9]{1,3}\.){3}[0-9]{1,3}|(?:(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9])\.)*(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9]))::(\d+)::SOCKET$").unwrap();
}

pub fn parse_visa_socket(captures: regex::Captures) -> Result<InstAddr, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
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
}
