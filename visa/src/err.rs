use std::convert::From;
use std::error::Error as ErrorTrait;
use std::fmt::{Display, Formatter, Result as FmtResult};
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
    PathNotMatchingLibrary(IoError),
    ///Null Character in path name
    NullCharacter,
    ///Unsupported Target
    UnsupportedPlatform,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use self::Error::*;
        f.write_str(&format!("{:?}", self))?;
        match self {
            OpeningLibraryError(msg) => {
                f.write_str(": ")?;
                msg.fmt(f)
            }
            SymbolGettingError(msg) => {
                f.write_str(": ")?;
                msg.fmt(f)
            }
            NullSymbol => f.write_str(": Symbol is Null."),
            PathNotMatchingLibrary(msg) => {
                f.write_str(": Path does not lead to a library.")?;
                msg.fmt(f)
            }
            NullCharacter => f.write_str(": The path contains a null character."),
            UnsupportedPlatform => f.write_str(": The target system is not supported by visa."),
        }
    }
}

impl From<dlopen::Error> for Error {
    fn from(value: dlopen::Error) -> Self {
        match value {
            dlopen::Error::NullCharacter(_) => Error::NullCharacter,
            dlopen::Error::OpeningLibraryError(e) => Error::OpeningLibraryError(e),
            dlopen::Error::SymbolGettingError(e) => Error::SymbolGettingError(e),
            dlopen::Error::NullSymbol => Error::NullSymbol,
            dlopen::Error::AddrNotMatchingDll(e) => Error::PathNotMatchingLibrary(e),
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            OpeningLibraryError(_) => "Could not open library",
            SymbolGettingError(_) => "Could not obtain symbol from the library",
            NullSymbol => "The symbol is NULL",
            PathNotMatchingLibrary(_) => "Address does not match any dynamic link library",
            NullCharacter => "Uncategorized",
            UnsupportedPlatform => "The target system is not supported by visa",
        }
    }

    fn cause(&self) -> Option<&dyn ErrorTrait> {
        use self::Error::*;
        match self {
            &OpeningLibraryError(_)
            | &SymbolGettingError(_)
            | &NullSymbol
            | &PathNotMatchingLibrary(_)
            | &NullCharacter
            | &UnsupportedPlatform => None,
        }
    }
}
