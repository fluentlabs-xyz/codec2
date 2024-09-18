use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),

    #[error("Decoding error: {0}")]
    Decoding(#[from] DecodingError),
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Not enough space in the buffer: required {required} bytes, but only {available} bytes available. {details}"
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

    #[error("Not enough data in the buffer: expected at least {expected} bytes, found {found}")]
    BufferTooSmall {
        expected: usize,
        found: usize,
        msg: String,
    },

    #[error("Unexpected end of buffer")]
    UnexpectedEof,

    #[error("Parsing error: {0}")]
    ParseError(String),
}
