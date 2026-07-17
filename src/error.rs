use std::fmt;

#[derive(Debug)]
pub struct VitError(pub String);

impl fmt::Display for VitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for VitError {}

impl From<std::io::Error> for VitError {
    fn from(err: std::io::Error) -> Self {
        VitError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, VitError>;
