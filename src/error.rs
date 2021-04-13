use thiserror::Error;

/// Error type returned by parser.
#[derive(Debug, Error)]
pub enum Error {
    /// An IO error occured while reading the data.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse a number.
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),

    /// The type of a data record is invalid.
    #[error("Invalid type: {0}")]
    InvalidType(u8),

    /// A invalid "Textkennzeichen" was read.
    #[error("Invalid Textkennzeichen: {0}")]
    InvalidTextkennzeichen(u8),

    /// Invalid Regionalschluessel
    #[error("Invalid Regionalschluessel: {0}")]
    ParseKey(#[from] ParseKeyError),
}

#[derive(Debug, Error)]
pub enum ParseKeyError {
    #[error("Key has invalid length: Expected {expected}, but got {got}: {s}")]
    InvalidLength {
        expected: usize,
        got: usize,
        s: String,
    },
    #[error("Keys must be numeric: {0}")]
    NonNumeric(String),
}

impl ParseKeyError {
    pub fn invalid_length(s: &str, expected: usize) -> Self {
        Self::InvalidLength {
            expected,
            got: s.len(),
            s: s.to_owned(),
        }
    }

    pub fn non_numeric(s: &str) -> Self {
        Self::NonNumeric(s.to_owned())
    }
}
