use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    ExpiredToken,
    AccessDenied,
    Other { description: String },
}

impl Error for AuthError {}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::ExpiredToken => write!(f, "Token has expired"),
            AuthError::AccessDenied => write!(f, "Access denied"),
            AuthError::Other { description } => write!(f, "Other error: {}", description),
        }
    }
}
