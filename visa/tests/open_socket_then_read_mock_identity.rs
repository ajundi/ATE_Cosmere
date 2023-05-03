use std::error::Error;
use std::ffi::CString;
use std::thread;

mod common;
use common::IPADDRESS;
use common::PORT;
use common::RETURN_MESSAGE;

#[test]
fn open_socket_then_read_mock_identity() -> Result<(), Box<dyn Error>> {
    let visa =
        visa::create(visa::Binary::Keysight).or_else(|_| visa::create(visa::Binary::Default))?;

    thread::spawn(common::run_mock_server);
    let mut _session = 0;
    visa.viOpenDefaultRM(&mut _session);
    assert_ne!(_session, 0, "When a session is open it is assigned a value that's not 0 depending on the visa implementation.");
    let address = CString::new(format!("TCPIP0::{IPADDRESS}::{PORT}::SOCKET"))?;
    let mut vi = 0;
    assert_eq!(
        visa.viOpen(_session, address.as_ptr(), 0, 0, &mut vi),
        0,
        "Visa Open Failed"
    );
    visa.viSetAttribute(vi, visa::VI_ATTR_TMO_VALUE, 5000); // Set timeout
    visa.viSetAttribute(vi, visa::VI_ATTR_TERMCHAR, 10); // set termination byte to 10
    visa.viSetAttribute(vi, visa::VI_ATTR_TERMCHAR_EN, 1); // enabled termination byte to stop reading when encountering this character.

    let mut ret_cnt: u32 = 0;
    let cmd = b"*IDN?\n";

    assert_eq!(
        visa.viWrite(vi, cmd.as_ptr(), u32::try_from(cmd.len())?, &mut ret_cnt,),
        0,
        "Failed to write to visa connection."
    );
    let resp = vec![0u8; 50];
    let status = visa.viRead(vi, resp.as_ptr() as *mut _, 50, &mut ret_cnt);
    let response = std::str::from_utf8(&resp[0..ret_cnt as usize])?;
    println!("Response : {}", response);
    assert!(ret_cnt > 0, "Failed to read with status: {status}");
    assert_eq!(
        &resp[0..ret_cnt as usize],
        RETURN_MESSAGE,
        "Returned message wasn't equal to what's expected."
    );
    assert_eq!(
        status,
        visa::VI_SUCCESS_TERM_CHAR as i32,
        "when using Sockets the transmission typically ends with the termination character."
    );
    visa.viClose(vi);
    Ok(())
}
