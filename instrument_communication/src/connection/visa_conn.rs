use crate::address::{InstAddr, VisaAddress, VisaType};
use crate::communication::InstConnection;
use crate::err::Error;
use crate::termination_bytes::TerminationBytes;
use dlopen::wrapper::Container;
use lazy_static::*;
use log::error;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
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
    visa: Arc<Container<VisaFuncs>>,
    bin: Binary,
    address: VisaAddress,
    buffer_size: usize,
    session: u32,
    term_string: Option<TerminationBytes>,
    frame_size: Option<usize>,
    is_term_char_attr_set: bool,
    timeout: Duration,
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

    pub fn connect(addr: VisaAddress, override_binary: Option<Binary>) -> Result<VisaConn, Error> {
        let binary = match override_binary {
            Some(b) => b,
            None => VisaConn::get_default_binary(),
        };
        let lib = try_load_binary(binary.clone())?;
        let mut vi = 0;
        match lib.0.viOpen(lib.1, addr.clone().into(), 0, 0, &mut vi) {
            status if status < 0 => {
                let msg = get_error_code(&lib.0, vi, status).unwrap_or("Failed to connect".into());
                return Err(Error::ConnectionFailed(msg));
            }
            _ => (),
        };
        match lib.0.viClear(vi) {
            status if status < 0 => {
                let msg = get_error_code(&lib.0, vi, status).unwrap_or("Failed to Clear, which indicate that most likely no usable instrument exists on this address even if it opens.".into());
                return Err(Error::ConnectionFailed(msg));
            }
            _ => (),
        };
        let mut visa_conn = VisaConn {
            visa: lib.0,
            bin: binary,
            address: addr,
            buffer_size: DEFAULT_BUFFER_SIZE,
            session: vi,
            term_string: None,
            frame_size: None,
            is_term_char_attr_set: false,
            timeout: Duration::from_secs(2),
        };
        visa_conn.set_termination(TerminationBytes::LF)?;
        Ok(visa_conn)
    }
    /// Checks if we should avoid enabling the term character attribute in the VISA driver.
    /// GPIB could have legacy equipment that sends binary data, which might have the
    /// termination character as a false positive. Moreover, GPIB has special signaling that
    /// indicates the end of transmission. Other types might also benefit from this.
    /// This setting only avoids checking termination for received data.
    fn should_avoid_term_char(visa_type: VisaType) -> bool {
        matches!(visa_type, VisaType::GPIB)
    }

    fn disable_term_char(&mut self) -> Result<(), Error> {
        Ok(
            match self
                .visa
                .viSetAttribute(self.session, visa::VI_ATTR_TERMCHAR_EN, 0)
            {
                status if status < 0 => {
                    let msg = get_error_code(&self.visa, self.session, status)
                        .unwrap_or_else(|| "Failed to disable termination char".into());
                    Err(Error::FunctionFailure(msg))?
                }
                _ => (),
            },
        )
    }
}

fn get_error_code(
    lib: &Arc<Container<VisaFuncs>>,
    vi: u32,
    status: i32,
) -> Option<Cow<'static, str>> {
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

fn try_load_binary(binary: Binary) -> Result<(Arc<Container<VisaFuncs>>, u32), Error> {
    let mut mutx_grd = match VISA_DICTIONARY.lock() {
        Ok(m) => m,
        Err(e) => {
            error!("{:?}", &e);
            e.into_inner()
        }
    };
    let visa = match mutx_grd.get(&binary) {
        Some(visa_lib) => visa_lib,
        None => {
            match visa::create(&binary) {
                Ok(lib) => {
                    let mut vi_session: u32 = 0;
                    let status = lib.viOpenDefaultRM(&mut vi_session);
                    if vi_session != 0 {
                        mutx_grd.insert(binary.clone(), Ok((Arc::new(lib), vi_session)));
                    } else {
                        mutx_grd.insert(binary.clone(), Err(Error::OpenSessionError(format!("visa session did not instantiate properly. visa dll exists but there might be a missing dependancy. status error code: {status}").into())));
                    }
                }
                Err(err) => {
                    mutx_grd.insert(
                        binary.clone(),
                        Err(Error::BinaryError(format!("{:?}", err).into())),
                    );
                }
            }
            mutx_grd.get(&binary).unwrap()
        }
    };
    match visa {
        Ok(v) => Ok(v.clone()),
        Err(e) => Err(e.clone()),
    }
}

impl InstConnection for VisaConn {
    fn address(&self) -> InstAddr {
        InstAddr::Visa(self.address.clone())
    }

    fn set_timeout(&self, timeout: Duration) -> Result<(), Error> {
        todo!()
    }

    fn reconnect(&mut self) -> Result<(), Error> {
        todo!()
    }

    fn set_termination(&mut self, term_bytes: TerminationBytes) -> Result<(), Error> {
        if VisaConn::should_avoid_term_char(self.address.get_type()) {
            self.disable_term_char()?;
        } else if let Some(last_byte) = term_bytes.bytes().last() {
            match self
                .visa
                .viSetAttribute(self.session, visa::VI_ATTR_TERMCHAR, *last_byte as u64)
            {
                status if status < 0 => {
                    let msg = get_error_code(&self.visa, self.session, status)
                        .unwrap_or_else(|| "Failed to set termination char".into());
                    Err(Error::FunctionFailure(msg))?
                }
                _ => (),
            }
            match self
                .visa
                .viSetAttribute(self.session, visa::VI_ATTR_TERMCHAR_EN, 1)
            {
                status if status < 0 => {
                    let msg = get_error_code(&self.visa, self.session, status)
                        .unwrap_or_else(|| "Failed to enable termination char".into());
                    Err(Error::FunctionFailure(msg))?
                }
                _ => (),
            }
        } else {
            self.disable_term_char()?;
        }
        self.term_string = Some(term_bytes);
        Ok(())
    }
}

#[test]
fn test_if_visa_not_installed_change_visa_socket_to_use_raw_socket() {
    let x = InstAddr::new("tcpip::localhost::5025::socket").unwrap();
    x.connect();
}
