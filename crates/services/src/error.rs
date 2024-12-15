use std::fmt;
use std::io;
use std::num::ParseIntError;

/// Enumeración para los posibles errores en el manejo de baterías.
#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    Zbus(zbus::Error),
    Icon(IconError),
    ParseError(ParseIntError),
    InvalidBatteryState(String),
    MissingBatteryField(String),
}

#[derive(Debug)]
pub enum IconError {
    IoError(io::Error),
    CharOrFileNotFound,
    CannotLoadFormats(&'static [&'static str]),
    CannotLoadFormat(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "I/O Error: {e}"),
            Error::ParseError(e) => write!(f, "Parse Error: {e}"),
            Error::Zbus(e) => write!(f, "Zbus Error: {e}"),
            Error::InvalidBatteryState(state) => {
                write!(f, "Invalid Battery State: {state}")
            }
            Error::MissingBatteryField(field) => {
                write!(f, "Missing Battery Field: {field}")
            }
            Error::Icon(e) => write!(f, "{e}"),
        }
    }
}

impl fmt::Display for IconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IconError::IoError(e) => write!(f, "I/O Error: {e}"),
            IconError::CharOrFileNotFound => write!(f, "Cannot find 'char' or 'path' to load icon"),
            IconError::CannotLoadFormat(e) => write!(f, "Cannot load icon from extension: {e}"),
            IconError::CannotLoadFormats(e) => write!(f, "Cannot load icon from extensions: {e:?}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Self::Zbus(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Self::ParseError(err)
    }
}

impl From<IconError> for Error {
    fn from(err: IconError) -> Self {
        Self::Icon(err)
    }
}
impl From<io::Error> for IconError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
