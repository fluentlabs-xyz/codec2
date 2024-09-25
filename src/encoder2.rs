use core::marker::PhantomData;

use byteorder::{BigEndian, ByteOrder};
use bytes::{Buf, BytesMut};

use crate::{
    encoder::{align, is_big_endian},
    error::{CodecError, DecodingError},
};

// pub type SolidityEncoderMode = EncoderMode<BE, 32, true>;
// pub type WasmEncoderMode = EncoderMode<LE, 4, false>;

pub trait HeaderSized {
    const HEADER_SIZE: usize;
}

/// Trait for encoding and decoding values with specific byte order, alignment, and mode.
///
/// # Type Parameters
/// - `B`: The byte order used for encoding/decoding.
/// - `ALIGN`: The alignment requirement for the encoded data.
/// - `SOL_MODE`: A boolean flag indicating whether Solidity-compatible mode is enabled.
pub trait Encoder<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>: Sized {
    /// Defines the header size for this encoder.
    ///
    /// We use an associated type implementing `HeaderSized` instead of a const
    /// `HEADER_SIZE` because traits with generic associated types (GATs) cannot
    /// have associated constants that depend on those types.
    type HeaderSize: HeaderSized;

    /// Returns the header size for this encoder.
    ///
    /// This method provides a convenient way to access the header size,
    /// which is defined by the associated `HeaderSize` type.
    fn header_size(&self) -> usize {
        Self::HeaderSize::HEADER_SIZE
    }

    /// Calculates the number of bytes needed to encode the value.
    ///
    /// This includes the header size and any additional space needed for alignment.
    /// The default implementation aligns the header size to the specified alignment.
    fn size_hint(&self) -> usize {
        align_up::<ALIGN>(self.header_size())
    }

    /// Encodes the value into the given buffer at the specified offset.
    ///
    /// # Arguments
    /// * `buf` - The buffer to encode into.
    /// * `offset` - The starting offset in the buffer for encoding.
    ///
    /// # Returns
    /// `Ok(())` if encoding was successful, or an error if encoding failed.
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError>;

    /// Decodes a value from the given buffer starting at the specified offset.
    ///
    /// # Arguments
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The starting offset in the buffer for decoding.
    ///
    /// # Returns
    /// The decoded value if successful, or an error if decoding failed.
    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError>;

    /// Partially decodes the header to determine the size and offset of the encoded data.
    ///
    /// # Arguments
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The starting offset in the buffer for decoding.
    ///
    /// # Returns
    /// A tuple `(size, data_offset)` where `size` is the total size of the encoded data
    /// and `data_offset` is the offset to the actual data (after the header).
    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError>;
}

pub struct SolidityABI<T>(PhantomData<T>);

impl<T> SolidityABI<T>
where
    T: Encoder<BigEndian, 32, true>,
{
    pub fn encode(value: &T, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        value.encode(buf, offset)
    }
}

#[inline]
pub const fn align_up<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}

impl HeaderSized for u8 {
    const HEADER_SIZE: usize = core::mem::size_of::<u8>();
}

impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, true> for u8 {
    type HeaderSize = u8;
    fn size_hint(&self) -> usize {
        align_up::<ALIGN>(Self::HEADER_SIZE as usize)
    }

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size = align_up::<ALIGN>(ALIGN.max(Self::HEADER_SIZE));

        if buf.len() < aligned_offset + word_size {
            buf.resize(aligned_offset + word_size, 0);
        }

        let aligned_value = align::<B, ALIGN, false>(&[*self]);
        buf[aligned_offset..aligned_offset + word_size].copy_from_slice(&aligned_value);
        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size = align_up::<ALIGN>(ALIGN.max(Self::HEADER_SIZE));

        if buf.remaining() < aligned_offset + word_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + word_size,
                found: buf.remaining(),
                msg: "buf too small to read aligned u8".to_string(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..];
        let value = if is_big_endian::<B>() {
            chunk[word_size - 1]
        } else {
            chunk[0]
        };

        Ok(value)
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        Ok((0, Self::HEADER_SIZE))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_up() {
        let value: u8 = 0x1;
        let mut buf = BytesMut::new();

        SolidityABI::encode(&value, &mut buf, 0).unwrap();
    }
}
