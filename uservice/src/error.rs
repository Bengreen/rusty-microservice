
/// Error type for handling errors on FFI calls
#[derive(Debug)]
pub enum UServiceError {
    Message(String),
    Unknown,
}


impl fmt::Display for UServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UServiceError::Message(_) => todo!(),
            UServiceError::Unknown => todo!(),
        }
    }
}

impl Error for UServiceError {}

// impl From<Box<dyn Any + Send + 'static>> for Error {
//    fn from(other: Box<dyn Any + Send + 'static>) -> Error {
//      if let Some(owned) = other.downcast_ref::<String>() {
//        Error::Message(owned.clone())
//      } else if let Some(owned) = other.downcast_ref::<String>() {
//        Error::Message(owned.to_string())
//      } else {
//        Error::Unknown
//      }
//    }
// }

use std::{fmt, error::Error};
