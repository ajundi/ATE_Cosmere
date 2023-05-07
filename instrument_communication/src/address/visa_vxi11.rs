use super::socket::resolve_ip;
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
    let ip_or_host = resolve_ip(&host_ip)?;
    return Ok(InstAddr::VisaVXI11(VisaAddress {
        address: format!("tcpip{}::{}::instr", board_num, ip_or_host),
        visa_type: PhantomData::<VXI11>,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::socket::HOSTNAME;
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
