use std::error::Error as ErrorTrait;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::convert::From;
use std::ffi::NulError;
use std::io::Error as IoError;


///This is a library-specific error that is returned by all calls to all APIs.
#[derive(Debug)]
pub enum Error {
    ///The library could not be opened.
    OpeningLibraryError(IoError),
    ///The symbol could not be obtained.
    SymbolGettingError(IoError),
    ///Value of the symbol was null.
    NullSymbol,
    ///Address could not be matched to a dynamic link library
    AddrNotMatchingDll(IoError),
    ///Uncategorized
    Uncategorized,
    ///Unsupported Target
    Unsupported,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::Error::*;
        f.write_str(&format!("{self}"))?;
        match self {
            &OpeningLibraryError(ref msg) => {
                f.write_str(": ")?;
                msg.fmt(f)
            },
            &SymbolGettingError(ref msg) => {
                f.write_str(": ")?;
                msg.fmt(f)
            },
            &NullSymbol => {
                f.write_str(": Symbol is Null.")
            },
            &AddrNotMatchingDll(ref msg) => {
                f.write_str(": Address Not Matching Dll")?;
                msg.fmt(f)
            },
            &Uncategorized => {
                f.write_str(": Uncategorized Error")
            },
            &Unsupported => {
                f.write_str(": Unsupported target.")
            },
        }
    }
}

impl From<dlopen::Error> for Error {
    fn from(value: dlopen::Error) -> Self {
        match value {
            dlopen::Error::NullCharacter(_) => { Error::Uncategorized }
            dlopen::Error::OpeningLibraryError(e) => { Error::OpeningLibraryError(e) }
            dlopen::Error::SymbolGettingError(e) => { Error::SymbolGettingError(e) }
            dlopen::Error::NullSymbol => { Error::NullSymbol }
            dlopen::Error::AddrNotMatchingDll(e) => { Error::AddrNotMatchingDll(e) }
        }
    }
}


impl ErrorTrait for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match self {
            &OpeningLibraryError(_) => "Could not open library",
            &SymbolGettingError(_) => "Could not obtain symbol from the library",
            &NullSymbol => "The symbol is NULL",
            &AddrNotMatchingDll(_) => "Address does not match any dynamic link library",
            &Uncategorized => "Uncategorized",
            &Unsupported => "The target system is not supported by visa"
        }
    }

    fn cause(&self) -> Option<&dyn ErrorTrait> {
        use self::Error::*;
        match self {
            &OpeningLibraryError(_) | &SymbolGettingError(_) | &NullSymbol | &AddrNotMatchingDll(_) | &Uncategorized | &Unsupported => {
                None
            }
        }
    }
}

