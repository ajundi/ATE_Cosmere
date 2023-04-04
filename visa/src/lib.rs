#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

mod err;
mod bindings;

use crate::err::Error;
use dlopen::wrapper::{Container, WrapperApi};
use dlopen_derive::WrapperApi;
use std::{io::Error as IoError, os::raw::c_float};
pub use bindings::*;


pub enum Binary {
    ///Keysight specific Visa binary which only exists if Keysight IO is installed
    Keysight,
    ///Generic visa binary this could be any vendor implementation. If visa is installed it has to exist.
    Generic,
    ///Custom path to a binary
    Custom(String),
}


pub fn create(lib: Binary) -> Result<Container<Wrapper>, Error> {
    let libPath:String=match lib {
        Binary::Keysight => if cfg!(target_family = "windows") {
            "ktvisa32.dll".into()
        }
        else if cfg!(target_family = "unix") && cfg!(target_pointer_width = "64"){
            "libiovisa.so".into()
        }
        else {return Err(Error::Unsupported);}
        Binary::Generic => if cfg!(target_family = "windows") {
            "visa32".into()
        }
        else if cfg!(target_family = "unix") && cfg!(target_pointer_width = "64"){
            "libvisa.so".into()
        }
        else if cfg!(target_family = "unix") && cfg!(target_pointer_width = "32"){
            "libvisa32.so".into()
        }
        else{return Err(Error::Unsupported);}
        Binary::Custom(path) =>path
    };
    unsafe {Container::load(libPath).map_err(|e| Error::from(e))}
}

#[cfg(test)]
mod tests {
    use super::Binary;
    use std::error::Error;
    use std::ffi::CString;

    #[test]
    fn failed_to_find_dll_file() {
        let lib = Binary::Custom("DummyLibraryThatDoesntExist".into());
        let visa = super::create(lib);
        if let Err(visa) = visa {
            println!("{:?}", visa);
            //println!("{}", visa)
        } else {
            assert!(false,)
        }
    }
}
