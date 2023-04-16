use std::{io, thread};
use std::time::Duration;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;

pub const RETURN_MESSAGE: &[u8; 34] = b"Cosmere,1234512,mock1000,V0.01.00\n";
pub const IPADDRESS: &str = "127.0.0.1";
pub const PORT: &str = "5030";

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

pub fn run_mock_server() -> io::Result<()> {
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