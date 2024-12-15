mod battery;

pub mod error;

pub use battery::*;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub struct ServiceManager;
