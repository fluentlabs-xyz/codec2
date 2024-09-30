use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Overflow error")]
    Overflow,
    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),

    #[error("Decoding error: {0}")]
    Decoding(#[from] DecodingError),
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Not enough space in the buf: required {required} bytes, but only {available} bytes available. {details}"
    )]
    BufferTooSmall {
        required: usize,
        available: usize,
        details: String,
    },

    #[error("Invalid data provided for encoding: {0}")]
    InvalidInputData(String),
}

#[derive(Debug, Error)]
pub enum DecodingError {
    #[error("Invalid data encountered during decoding: {0}")]
    InvalidData(String),

    #[error("Not enough data in the buf: expected at least {expected} bytes, found {found}")]
    BufferTooSmall {
        expected: usize,
        found: usize,
        msg: String,
    },

    #[error("Buffer overflow: {msg}")]
    BufferOverflow { msg: String },

    #[error("Unexpected end of buf")]
    UnexpectedEof,

    #[error("Overflow error")]
    Overflow,

    #[error("Parsing error: {0}")]
    ParseError(String),
}
