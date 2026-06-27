use std::fmt;

#[derive(Debug)]
pub enum VitError {
    Io(std::io::Error),
    CorruptIndex { file: &'static str, detail: String },
    NoCommits,
    InsufficientData,
}

impl fmt::Display for VitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VitError::Io(err) => write!(f, "{}", err),
            VitError::CorruptIndex { file, detail } => {
                write!(f, "corrupt index '{}': {}", file, detail)
            }
            VitError::NoCommits => write!(f, "no commits found"),
            VitError::InsufficientData => {
                write!(f, "not enough data for LSA")
            }
        }
    }
}

impl std::error::Error for VitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VitError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for VitError {
    fn from(err: std::io::Error) -> Self {
        VitError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, VitError>;
