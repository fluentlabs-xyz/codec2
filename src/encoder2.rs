use core::marker::PhantomData;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::{Buf, BytesMut};

use crate::{
    encoder::{align, is_big_endian},
    error::{CodecError, DecodingError},
};

/// Trait for encoding and decoding values with specific byte order, alignment, and mode.
///
/// # Type Parameters
/// - `B`: The byte order used for encoding/decoding.
/// - `ALIGN`: The alignment requirement for the encoded data.
/// - `SOL_MODE`: A boolean flag indicating whether Solidity-compatible mode is enabled.
pub trait Encoder<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>: Sized {
    /// Returns the header size for this encoder.
    const HEADER_SIZE: usize;

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

    /// Partially decodes the header to determine the length and offset of the encoded data.
    ///
    /// # Arguments
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The starting offset in the buffer for decoding.
    ///
    /// # Returns
    /// A tuple `(data_offset, data_length)` if successful, or an error if decoding failed.
    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError>;

    /// Calculates the number of bytes needed to encode the value.
    ///
    /// This includes the header size and any additional space needed for alignment.
    /// The default implementation aligns the header size to the specified alignment.
    fn size_hint(&self) -> usize {
        align_up::<ALIGN>(Self::HEADER_SIZE)
    }
}

#[inline]
pub const fn align_up<const ALIGN: usize>(val: usize) -> usize {
    (val + ALIGN - 1) & !(ALIGN - 1)
}

macro_rules! define_encoder_mode {
    ($name:ident, $byte_order:ty, $align:expr, $sol_mode:expr) => {
        pub struct $name<T>(PhantomData<T>);

        impl<T> $name<T>
        where
            T: Encoder<$byte_order, $align, $sol_mode>,
        {
            pub fn encode(value: &T, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
                value.encode(buf, offset)
            }

            pub fn decode(buf: &impl Buf, offset: usize) -> Result<T, CodecError> {
                T::decode(buf, offset)
            }

            pub fn partial_decode(
                buf: &impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                T::partial_decode(buf, offset)
            }

            pub fn size_hint(value: &T) -> usize {
                value.size_hint()
            }
        }
    };
}

// Define encoder modes for Solidity and Wasm ABI
define_encoder_mode!(SolidityABI, BigEndian, 32, true);
define_encoder_mode!(WasmABI, LittleEndian, 4, false);

// Example of implementing the Encoder trait for u8
impl<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool> Encoder<B, ALIGN, SOL_MODE> for u8 {
    const HEADER_SIZE: usize = core::mem::size_of::<u8>();

    fn size_hint(&self) -> usize {
        align_up::<ALIGN>(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE)
    }
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size =
            align_up::<ALIGN>(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));

        if buf.len() < aligned_offset + word_size {
            buf.resize(aligned_offset + word_size, 0);
        }

        let aligned_value = align::<B, ALIGN, false>(&[*self]);
        buf[aligned_offset..aligned_offset + word_size].copy_from_slice(&aligned_value);
        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size =
            align_up::<ALIGN>(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));

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

    fn partial_decode(_buf: &impl Buf, _offset: usize) -> Result<(usize, usize), CodecError> {
        Ok((0, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_align_up() {
        let value: u8 = 0x1;

        // Solidity ABI
        let mut buf = BytesMut::new();
        SolidityABI::encode(&value, &mut buf, 0).unwrap();
        assert_eq!(
            "0000000000000000000000000000000000000000000000000000000000000001",
            hex::encode(&buf.chunk()[..])
        );
        let decoded_sol = SolidityABI::<u8>::decode(&buf, 0).unwrap();

        // Wasm ABI
        let mut buf = BytesMut::new();
        WasmABI::encode(&value, &mut buf, 0).unwrap();
        assert_eq!("01000000", hex::encode(&buf.chunk()[..]));

        let decoded_wasm = WasmABI::<u8>::decode(&buf, 0).unwrap();

        // Ensure that the decoded values are the same
        assert_eq!(decoded_sol, decoded_wasm);
    }
}
