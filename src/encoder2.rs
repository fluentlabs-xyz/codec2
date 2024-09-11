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

pub trait Encoder: Sized + Default {
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
        // Align the offset and header size
        let aligned_offset = align::<ALIGN>(offset);
        let aligned_header_size = align::<ALIGN>(Self::HEADER_SIZE);

        // Check if the buffer is large enough
        if buf.remaining_mut() < aligned_offset + aligned_header_size {
            return Err(EncoderError::BufferTooSmall);
        }

        // Encode the value
        self.encode_inner::<B, ALIGN>(buf, aligned_offset)
    }

    /// Encodes the value into the given buffer at the specified aligned offset.
    ///
    /// Note: This method is called internally by the `encode` function after the buffer has been checked for
    /// sufficient size and the offset has been aligned. It does **not** perform any buffer size validation.
    /// The caller must ensure that the buffer is large enough before calling this method.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer into which the value will be encoded.
    /// * `offset` - The aligned offset in the buffer to start encoding at.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encoding was successful, or an `EncoderError` if there was an issue during encoding.
    ///
    /// This method should be overridden if additional operations are required before encoding the value.
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
