use std::fmt::{Formatter};

#[derive(Debug)]
pub enum ParsingError {
    UnsupportedOrMissingMethodError(String),
    NoPathError,
    UnsupportedHeaderFormat(String),
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedOrMissingMethodError(s) => write!(f, "Failed to parse http method: {}", s),
            Self::NoPathError => write!(f, "Failed to parse path"),
            Self::UnsupportedHeaderFormat(s) => write!(f, "Unsupported header format: {}", s)
        }
    }
}

impl std::error::Error for ParsingError {}