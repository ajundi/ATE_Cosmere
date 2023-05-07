use address::InstAddr;
use communication::InstConnection;
use err::Error;

pub mod address;
pub mod communication;
pub mod err;

/// Open a connection to an address provided as a simple string. This simplifies the process of creating
/// an address object first then opening the connection. This is yet to mature as the API stabilizes.
pub fn open_connection<T:AsRef<str>>(address: T) -> Result<Box<dyn InstConnection>, Error> {
    let _address = address.as_ref().parse::<InstAddr>()
        .or_else(|msg| Err(Error::Other(format!("Failed to create address. Error: {msg}").to_owned())))?;
    _address.open_connection()
    // todo!()
}
