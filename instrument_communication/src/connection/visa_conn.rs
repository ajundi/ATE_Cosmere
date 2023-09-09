use crate::address::{InstAddr, VisaAddress};
use crate::communication::InstConnection;
use crate::err::Error;
use crate::termination_string::TerminationString;
use dlopen::wrapper::Container;
use lazy_static::*;
use log::error;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use visa::*;
const MAXIMUM_BUFFER_SIZE: usize = 50000000;
const DEFAULT_BUFFER_SIZE: usize = 4096;
const ERR_MSG_BUFFER_SIZE: usize = 512;

lazy_static! {
    pub static ref DEFAULT_BINARY: Mutex<Binary> = Mutex::new(Binary::Primary);
    static ref VISA_DICTIONARY: Mutex<HashMap<Binary, Result<(Arc<Container<VisaFuncs>>, ViSession), Error>>> =
        Mutex::new(HashMap::new());
}
pub struct VisaConn {
    bin: Arc<Container<VisaFuncs>>,
    address: InstAddr,
    buffer_size: usize,
    session: u32,
    term_string: TerminationString,
    frame_size: usize,
    is_term_char_attr_set: bool,
    timeout: i32,
}

impl VisaConn {
    ///
    pub fn set_default_binary(binary: Binary) {
        let mut guard = DEFAULT_BINARY.lock().unwrap();
        *guard = binary;
    }

    fn get_default_binary() -> Binary {
        match DEFAULT_BINARY.lock() {
            Ok(bin) => (*bin).clone(),
            Err(_) => Binary::default(),
        }
    }

    pub fn connect<T>(
        addr: VisaAddress<T>,
        override_binary: Option<Binary>,
    ) -> Result<VisaConn, Error> {
        let mut binary = match override_binary {
            Some(b) => b,
            None => VisaConn::get_default_binary(),
        };
        let lib = try_load_binary(binary)?;
        let mut vi = 0;
        match lib.0.viOpen(lib.1, addr.into(), 0, 0, &mut vi) {
            status if status >= 0 => (),
            status => {
                let msg = get_error_code(lib.0, vi, status).unwrap_or("Failed to connect".into());
                return Err(Error::ConnectionFailed(msg));
            }
        };
        match lib.0.viClear(vi) {
            status if status >= 0 => (),
            status => {
                let msg = get_error_code(lib.0, vi, status).unwrap_or("Failed to Clear, which indicate that most likely no usable instrument exists on this address even if it opens.".into());
                return Err(Error::ConnectionFailed(msg));
            }
        };
        Ok(VisaConn{
            bin: lib.0,
            address: todo!(),
            buffer_size: DEFAULT_BUFFER_SIZE,
            session: vi,
            term_string: todo!(),
            frame_size: todo!(),
            is_term_char_attr_set: todo!(),
            timeout: todo!(),
        })
    }
}

fn get_error_code(lib: Arc<Container<VisaFuncs>>, vi: u32, status: i32) -> Option<Cow<'static,str>> {
    let mut resp = vec![0u8; ERR_MSG_BUFFER_SIZE];
    match lib.viStatusDesc(vi, status, resp.as_mut_ptr()) {
        0 => {
            if let Ok(msg) = std::str::from_utf8(&resp) {
                Some(msg.to_owned().into())
            } else {
                None
            }
        }
        n => None,
    }
}

fn try_load_binary<'a>(binary: Binary) -> Result<(Arc<Container<VisaFuncs>>, u32), Error> {
    let mut mutx_grd = match VISA_DICTIONARY.lock() {
        Ok(m) => m,
        Err(e) => {
            error!("{:?}", &e);
            e.into_inner()
        }
    };
    let bin = binary.clone();
    let visa = match mutx_grd.get(&binary) {
        Some(visa_lib) => visa_lib,
        None => {
            match visa::create(&binary) {
                Ok(lib) => {
                    let mut viSession: u32 = 0;
                    let status = lib.viOpenDefaultRM(&mut viSession);
                    if viSession != 0 {
                        mutx_grd.insert(binary, Ok((Arc::new(lib), viSession)));
                    } else {
                        mutx_grd.insert(binary, Err(Error::OpenSessionError(format!("visa session did not instantiate properly. visa dll exists but there might be a missing dependancy. status error code: {status}").into())));
                    }
                }
                Err(err) => {
                    mutx_grd.insert(binary, Err(Error::BinaryError(format!("{:?}", err).into())));
                }
            }
            mutx_grd.get(&bin).unwrap()
        }
    };
    match visa {
        Ok(v) => Ok(v.clone()),
        Err(e) => Err(e.clone()),
    }
}

impl InstConnection for VisaConn {
    fn address(&self) -> InstAddr {
        todo!()
    }

    fn set_timeout(&self, timeout: u64) -> Result<(), crate::err::Error> {
        todo!()
    }

    fn reconnect(&self) -> Result<(), crate::err::Error> {
        todo!()
    }
}

#[test]
fn test_if_visa_not_installed_change_visa_socket_to_use_raw_socket() {
    let x = InstAddr::new("tcpip::localhost::5025::socket").unwrap();
    x.connect();
}
