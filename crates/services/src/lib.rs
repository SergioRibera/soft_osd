mod battery;

pub mod error;

pub use battery::*;
pub use error::Error;

pub type ServiceResult<T> = Result<T, Error>;
