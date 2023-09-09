use crate::address::*;

lazy_static! {
    pub static ref GPIB_ADDRESS_REGEX: Regex =
        Regex::new(r"^(?i)GPIB(\d*)::(\d+)(?:::\d+)?(?:::INSTR)?$").unwrap();
}
pub fn parse_gpib(captures: regex::Captures) -> Result<InstAddr, String> {
    let instr_num = captures[2].to_string();
    if instr_num.len() < 3 && instr_num.cmp(&"31".to_owned()) == std::cmp::Ordering::Less {
        let board_num = if captures[1].len() == 0 {
            "0".to_owned()
        } else {
            captures[1].to_string()
        };
        let address = format!("gpib{}::{}::instr", board_num, instr_num);
        return Ok(InstAddr::Visa(VisaAddress {
            address,
            visa_type: VisaType::GPIB,
        }));
    } else {
        Err(format!("Invalid primary GPIB address {}", instr_num))
    }
}

impl VisaAddress {
    #[allow(dead_code)]
    fn gpib_complement_address(&self) -> Option<String> {
        match self.visa_type {
            VisaType::GPIB => {
                let address_parts: Vec<&str> = self.address.split("::").collect();
                let mut num = address_parts[1].parse::<i32>().unwrap();
                num += (num % 2 == 0).then(|| 1).unwrap_or(-1);
                Some(format!(
                    "{}::{}::{}",
                    address_parts[0], num, address_parts[2]
                ))
            }
            _ => None,
        }
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
            visa_type: VisaType::GPIB,
        };
        assert_eq!(&address.gpib_complement_address().unwrap(), expected);
    }

    #[test_case("GPIB0::18::INSTR", "GPIB0::17::INSTR")]
    #[test_case("GPIB0::15::INSTR", "GPIB0::13::INSTR")]
    #[test_case("GPIB1::16::INSTR", "GPIB0::17::INSTR")]
    #[test_case("GPIB1::16::INSTR", "GPIB0::17::INST")]
    #[test_case("GPIB1::16::INSTR", "GPIB0::17::INSTD")]
    fn test_gpib_complement_address_fails(original: &str, expected: &str) {
        let address = VisaAddress {
            address: String::from(original),
            visa_type: VisaType::GPIB,
        };
        assert!(!address
            .gpib_complement_address().unwrap()
            .eq_ignore_ascii_case(expected));
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
}
