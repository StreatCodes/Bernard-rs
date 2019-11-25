use std::{convert, io, error, fmt};

#[derive(Debug)]
pub struct Error {
    message: String
}

impl Error {
    pub fn new(message: String) -> Error {
        Error {
            message: message
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str { &self.message }
}

impl convert::From<std::io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::new(error.to_string())
    }
}

// impl convert::From<option::NoneError> for Error {
//     fn from(error: option::NoneError) -> Self {
//         Error::new(error.to_string())
//     }
// }

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}