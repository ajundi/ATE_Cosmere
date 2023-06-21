use address::InstAddr;
use communication::InstConnection;
use err::Error;

pub mod address;
pub mod communication;
pub mod err;

/// Open a connection to an address provided as a simple string. This simplifies the process of creating
/// an address object first then opening the connection. This is yet to mature as the API stabilizes.
pub fn connect<T: AsRef<str>>(address: T) -> Result<Box<dyn InstConnection>, Error<'static>> {
    let _address = InstAddr::new(address).or_else(|msg| {
        Err(Error::ParseFailed(
            format!("Failed to create address. Error: {msg}").into(),
        ))
    })?;
    _address.connect()
    // todo!()
}
