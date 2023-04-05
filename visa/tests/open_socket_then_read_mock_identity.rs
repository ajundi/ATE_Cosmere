use std::error::Error;
use std::ffi::CString;

use std::{io, thread};
use std::time::Duration;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;

const RETURN_MESSAGE: &[u8; 3] = b"Hi\n";
const IPADDRESS: &str = "127.0.0.1";
const PORT: &str = "5030";

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    loop {
        let mut buf = [0; 1024];
        if let Ok(n) = stream.read(&mut buf) {
            if n == 0 {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            let msg = String::from_utf8_lossy(&buf[..n]);
            println!("Received: {:?}", msg);
            if msg.to_lowercase().contains("*idn?") {
                thread::sleep(Duration::from_secs(1));
                stream.write_all(RETURN_MESSAGE)?;
            }
        }
    }
}

fn run_mock_server() -> io::Result<()> {
    let listener = TcpListener::bind(format!("{IPADDRESS}:{PORT}"))?;
    println!("Server listening on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            thread::spawn(|| handle_client(stream).unwrap_or_else(|e| eprintln!("Error handling client: {:?}", e)));
        } else if let Err(e) = stream {
            eprintln!("Error accepting client: {:?}", e);
        }
    }

    Ok(())
}

#[test]
fn open_socket_then_read_mock_identity() -> Result<(), Box<dyn Error>> {
        let visa = visa::create(visa::Binary::Keysight)
        .unwrap_or(visa::create(visa::Binary::Default)?);

    thread::spawn(run_mock_server);
    let mut _session = 0;
    visa.viOpenDefaultRM(&mut _session);
    assert_ne!(_session, 0, "When a session is open it is assigned a value that's not 0 depending on the visa implementation.");
    let address2 = CString::new(format!("TCPIP0::{IPADDRESS}::{PORT}::SOCKET"))?;
    let mut vi = 0;
    assert_eq!(visa.viOpen(_session, address2.as_ptr(), 0, 0, &mut vi), 0, "Visa Open Failed");
    visa.viSetAttribute(vi, visa::VI_ATTR_TMO_VALUE, 5000);
    visa.viSetAttribute(vi, visa::VI_ATTR_TERMCHAR, 10);
    visa.viSetAttribute(vi, visa::VI_ATTR_TERMCHAR_EN, 1);

    let mut ret_cnt: u32 = 0;
    let cmd = b"*IDN?\n";

    assert_eq!(visa.viWrite(
        vi,
        cmd.as_ptr(),
        u32::try_from(cmd.len())?,
        &mut ret_cnt,
    ), 0, "Failed to write to visa connection.");
    let resp = vec![0u8; 50];
    let status = visa.viRead(vi, resp.as_ptr() as *mut _, 50, &mut ret_cnt);
    let response = std::str::from_utf8(&resp[0..ret_cnt as usize])?;
    println!("Return data : {:?}", response);
    assert!(ret_cnt > 0, "Failed to read with status: {status}");
    assert_eq!(&resp[0..ret_cnt as usize], RETURN_MESSAGE, "Returned message wasn't equal to what's expected.");
    assert_eq!(status, visa::VI_SUCCESS_TERM_CHAR as i32, "when using Sockets the transmission typically ends with the termination character.");
    visa.viClose(vi);
    Ok(())
}