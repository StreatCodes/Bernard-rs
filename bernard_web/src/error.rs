use std::{convert, io, error, fmt};

#[derive(Debug)]
pub struct BernardError {
    message: String
}

impl BernardError {
    pub fn new(message: String) -> BernardError {
        BernardError {
            message: message
        }
    }
}

impl error::Error for BernardError {
    fn description(&self) -> &str { &self.message }
}

impl convert::From<std::io::Error> for BernardError {
    fn from(error: io::Error) -> Self {
        BernardError::new(error.to_string())
    }
}

// impl convert::From<option::NoneError> for BernardError {
//     fn from(error: option::NoneError) -> Self {
//         BernardError::new(error.to_string())
//     }
// }

impl std::fmt::Display for BernardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}