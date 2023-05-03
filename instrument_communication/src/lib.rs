use communication::InstConnection;
use err::Error;

pub mod address;
pub mod communication;
pub mod err;

/// Open a connection to an address provided as a simple string. This simplifies the process of creating
/// an address object first then opening the connection. This is yet to mature as the API stabilizes. 
pub fn open_connection(address: &dyn AsRef<str>) -> Result<Box<dyn InstConnection>, Error> {
    let _address = address::create(address)
        .ok_or_else(|| Error::Other("Failed to create address".to_owned()))?;
    _address.open_connection()
}

