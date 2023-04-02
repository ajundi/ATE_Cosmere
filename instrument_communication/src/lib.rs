
pub enum Error {
    Timeout,
    Other(String),
}


pub trait Communication {
    fn connect(&self) -> Result<(), Error>;

}
