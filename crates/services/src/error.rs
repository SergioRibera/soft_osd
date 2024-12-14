use std::fmt;
use std::io;
use std::num::ParseIntError;

/// Enumeración para los posibles errores en el manejo de baterías.
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ParseError(ParseIntError),
    InvalidBatteryState(String),
    MissingBatteryField(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "I/O Error: {}", e),
            Error::ParseError(e) => write!(f, "Parse Error: {}", e),
            Error::InvalidBatteryState(state) => {
                write!(f, "Invalid Battery State: {}", state)
            }
            Error::MissingBatteryField(field) => {
                write!(f, "Missing Battery Field: {}", field)
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseError(err)
    }
}
