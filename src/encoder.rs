use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::{Buf, BytesMut};

use thiserror::Error;

/// ByteOrderExt is a trait that extends the functionality of the `ByteOrder` trait. It provides a method to determine if the byte order is big endian.
pub trait ByteOrderExt: ByteOrder {
    fn is_big_endian() -> bool;
}

impl ByteOrderExt for BigEndian {
    fn is_big_endian() -> bool {
        true
    }
}

impl ByteOrderExt for LittleEndian {
    fn is_big_endian() -> bool {
        false
    }
}
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("Encoding error: {0}")]
    Encoding(#[from] EncodingError),

    #[error("Decoding error: {0}")]
    Decoding(#[from] DecodingError),
}

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("Not enough space in the buffer: required {required} bytes, but only {available} bytes available. {details}")]
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
    BufferTooSmall { expected: usize, found: usize },

    #[error("Unexpected end of buffer")]
    UnexpectedEof,

    #[error("Parsing error: {0}")]
    ParseError(String),
}

// TODO: @d1r1 Investigate whether decoding the result into an uninitialized memory (e.g., using `MaybeUninit`)
// would be more efficient than initializing with `Default`.
// This could potentially reduce unnecessary memory initialization overhead in cases where
// the default value is not required before the actual decoding takes place.
// Consider benchmarking both approaches to measure performance differences.

pub trait Encoder: Sized + Default {
    /// Header used to save metadata about the encoded value.
    const HEADER_SIZE: usize;

    /// Returns known size of the encoded value data.
    const DATA_SIZE: usize;

    /// How many bytes we should allocate for the encoded value.
    /// This is the sum of the header size and the known data size.
    fn size_hint<const ALIGN: usize>(&self) -> usize {
        round_up_to_alignment::<ALIGN>(Self::HEADER_SIZE)
            + round_up_to_alignment::<ALIGN>(Self::DATA_SIZE)
    }

    /// Encodes the value into the given buffer at the specified offset. The buffer must be large enough to hold at least `align(offset) + Self::HEADER_SIZE` bytes.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to encode into.
    /// * `offset` - The offset in the buffer to start encoding at.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encoding was successful, or an `EncoderError` if there was a problem.
    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError>;

    /// Decodes a value from the given buffer starting at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns the decoded value if successful, or an `EncoderError` if there was a problem.
    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError>;

    /// Decodes the header to determine the size of the encoded data and offset to the data.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(offset, data_length)` if successful, or an `EncoderError` if there was a problem.
    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError>;
}

/// Rounds up the given offset to the nearest multiple of ALIGN.
/// ALIGN must be a power of two.
#[inline]
pub const fn round_up_to_alignment<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}
