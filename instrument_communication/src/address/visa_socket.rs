use crate::address::*;

lazy_static! {
    pub static ref VISASOCKET_ADDRESS_REGEX: Regex =
        Regex::new(r"^(?i)TCPIP(\d*)::([a-z0-9\.]+)::(\d+)::SOCKET$").unwrap();
}

pub fn parse_visa_socket(captures: regex::Captures) -> Result<InstAddr, String> {
    let board_num = captures[1].to_string();
    let board_num = if board_num.is_empty() {
        "0".into()
    } else {
        board_num
    };
    let host_ip = captures[2].to_string();
    let socket: Socket = format!("{}:{}", host_ip, &captures[3]).parse()?;
    Ok(InstAddr::Visa(VisaAddress {
        address: format!(
            "tcpip{}::{}::{}::socket",
            board_num,
            socket.ip_or_host(),
            socket.port()
        ),
        visa_type: VisaType::Socket,
    }))
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

    #[test_case("TCPIP0 :: 256.168.0.1::5025:: SockEt ";"Invalid IP Address is interpreted as Host name as raw socket address")]
    fn test_visa_socket_invalid_address_is_a_valid_host_name(address: &str) {
        let inst_address = address.parse::<InstAddr>();
        assert!(inst_address.is_ok());
    }
}
