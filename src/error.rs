use thiserror::Error;
use xcb::*;
pub type Result<T> = std::result::Result<T, Error>;

//TODO: better errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("xcb connection failed")]
    Connection(#[from] xcb::ConnError),
    #[error("xcb error")]
    Xcb(#[from] xcb::Error),
    #[error("xcb error")]
    Protocol(#[from] xcb::ProtocolError),
    #[error("unknown")]
    Unknown,
}
