use byteorder::ByteOrder;
use bytes::{Buf, BufMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Not enough space in the buffer")]
    BufferTooSmall,
    #[error("Invalid data encountered during decoding")]
    InvalidData,
    #[error("Unexpected end of buffer")]
    UnexpectedEof,
}

pub trait Encoder: Sized {
    /// The size of the header for this encodable type.
    const HEADER_SIZE: usize;

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
    fn encode<B: ByteOrder, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError> {
        let aligned_offset = align::<ALIGN>(offset);
        if buf.remaining_mut() < aligned_offset + Self::HEADER_SIZE {
            return Err(EncoderError::BufferTooSmall);
        }

        self.encode_inner::<B, ALIGN>(buf, aligned_offset)
    }

    /// Encodes the value into the given buffer at the specified offset.
    /// This method is called after the buffer has been aligned.
    /// The default implementation calls `encode` after aligning the offset.
    /// Override this method if you need to perform additional operations before encoding.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to encode into.
    /// * `offset` - The aligned offset in the buffer to start encoding at.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encoding was successful, or an `EncoderError` if there was a problem.
    fn encode_inner<B: ByteOrder, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError>;

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
    fn decode<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError>;

    /// Decodes the header to determine the size of the encoded data.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(offset, data_length)` if successful, or an `EncoderError` if there was a problem.
    fn partial_decode<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError>;
}

#[inline]
pub const fn align<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}
