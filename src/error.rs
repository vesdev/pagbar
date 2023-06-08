use thiserror::Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;
//TODO: better errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown")]
    Unknown,
}
