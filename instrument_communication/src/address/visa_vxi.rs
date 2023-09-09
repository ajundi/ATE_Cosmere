use super::socket::NetworkAddr;
use crate::address::*;

lazy_static! {
    pub static ref VISAVXI11_ADDRESS_REGEX: Regex =
     Regex::new(r"^(?i)TCPIP(\d*)::((?:[0-9]{1,3}\.){3}[0-9]{1,3}|(?:(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9])\.)*(?:[a-z]|[a-z][a-z0-9\-]*[a-z0-9]))(?:::INSTR)?$").unwrap();
}

pub fn parse_visa_vxi11(captures: regex::Captures) -> Result<InstAddr, String> {
    let board_num = captures[1].to_string();
    let board_num = if board_num.len() == 0 {
        "0".to_owned()
    } else {
        board_num
    };
    let host_ip = captures[2].to_string();
    let ip_or_host =  NetworkAddr::from_str(&host_ip)?;
    return Ok(InstAddr::Visa(VisaAddress {
        address: format!("tcpip{}::{}::instr", board_num, ip_or_host),
        visa_type: VisaType::VXI,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::socket::LOCAL_MACHINE;
    use test_case::test_case;

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
        let inst_address = format!("TCPIP::{}::INSTR", LOCAL_MACHINE.as_str()).parse::<InstAddr>();
        assert!(inst_address.is_ok());
        assert!(inst_address
            .unwrap()
            .address()
            .eq_ignore_ascii_case("tcpip0::127.0.0.1::INSTR"));
    }

    #[test_case("TCPIP1::google.com")]
    #[test_case("TCPIP2::amazon.com::INSTR")]
    #[test_case("TCPIP3::mail.example.com")]
    #[test_case("TCPIP4::ftp.test-site.com")]
    #[test_case("TCPIP5::test-site.com")]
    #[test_case("TCPIP6::localhost::INSTR")]
    #[test_case("TCPIP7::127.0.0.1")]
    #[test_case("TCPIP8::a-very-long-hostname-with-numbers-123.com")]
    #[test_case("TCPIP9::mail.example.co.uk")]
    #[test_case("TCPIP10::a-host-name-with-dashes.com")]
    #[test_case("TCPIP11::example.co.uk")]
    #[test_case("TCPIP12::127.0.0.1::INSTR")]
    #[test_case("TCPIP13::a.b.c.d")]
    #[test_case("TCPIP14::a.b-c.d")]
    #[test_case("TCPIP15::www.example.com")]
    #[test_case("TCPIP16::a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z")]
    #[test_case("TCPIP17::test.example.co.uk")]
    #[test_case("TCPIP18::a.host-name.with.dots.com")]
    #[test_case("TCPIP19::a.b.c")]
    fn test_hostname_regex(address: &str) {
        assert!(VISAVXI11_ADDRESS_REGEX.is_match(address));
    }
}
