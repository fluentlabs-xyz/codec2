use std::usize;

use crate::encoder::{
    align_up, read_u32_aligned, write_u32_aligned, ByteOrderExt, CodecError, DecodingError, Encoder,
};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};

use bytes::{Buf, BytesMut};

const DEFAULT_HEADER_ELEM_SIZE: usize = 4;

// Write bytes to the buffer
// Returns the size of the header
// To avoid resizing buffer, you can pre-allocate the buffer with the size of the header before calling this function
// The header contains the offset and length of the data
// The actual data is appended to the buffer, after the header
pub fn write_bytes<B: ByteOrderExt, const ALIGN: usize>(
    buffer: &mut BytesMut,
    offset: usize,
    bytes: &[u8],
) -> usize {
    let aligned_offset = align_up::<ALIGN>(offset);

    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);
    let aligned_header_size = aligned_elem_size * 2;

    if buffer.len() < aligned_offset + aligned_header_size {
        buffer.resize(aligned_offset + aligned_header_size, 0);
    }
    // We append the data to the buffer. So the offset of the data is the current length of the buffer
    let data_offset = buffer.len();

    // Write header
    write_u32_aligned::<B, ALIGN>(buffer, aligned_offset, data_offset as u32);

    // Write length of the data
    write_u32_aligned::<B, ALIGN>(
        buffer,
        aligned_offset + aligned_elem_size,
        bytes.len() as u32,
    );

    // Append data
    buffer.extend_from_slice(bytes);

    aligned_header_size
}

pub fn read_bytes<B: ByteOrderExt, const ALIGN: usize>(buffer: &impl Buf, offset: usize) -> Bytes {
    let (data_offset, data_len) = read_bytes_header::<B, ALIGN>(buffer, offset);

    let data = buffer.chunk()[data_offset..data_offset + data_len].to_vec();

    Bytes::from(data)
}

pub fn read_bytes_header<B: ByteOrderExt, const ALIGN: usize>(
    buffer: &impl Buf,
    offset: usize,
) -> (usize, usize) {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);

    let data_offset = read_u32_aligned::<B, ALIGN>(buffer, aligned_offset) as usize;
    let data_len =
        read_u32_aligned::<B, ALIGN>(buffer, aligned_offset + aligned_elem_size) as usize;

    (data_offset, data_len)
}

impl Encoder for Bytes {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;
    const DATA_SIZE: usize = 0;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buffer: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let _ = write_bytes::<B, ALIGN>(buffer, offset, self);
        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buffer: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        Ok(read_bytes::<B, ALIGN>(buffer, offset))
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        Ok(read_bytes_header::<B, ALIGN>(buf, offset))
    }
}

impl<const N: usize> Encoder for FixedBytes<N> {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = N;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buffer: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        if buffer.len() < offset + N {
            buffer.resize(offset + N, 0);
        }
        buffer[offset..offset + N].copy_from_slice(self.as_ref());
        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buffer: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let data = buffer.chunk()[offset..offset + N].to_vec();
        Ok(FixedBytes::from_slice(&data))
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        _buffer: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        Ok((offset, N))
    }
}

macro_rules! impl_evm_fixed {
    ($typ:ty) => {
        impl Encoder for $typ {
            const HEADER_SIZE: usize = 0;
            const DATA_SIZE: usize = <$typ>::len_bytes();

            fn encode<B: ByteOrderExt, const ALIGN: usize>(
                &self,
                buffer: &mut BytesMut,
                offset: usize,
            ) -> Result<(), CodecError> {
                self.0.encode::<B, ALIGN>(buffer, offset)
            }

            fn decode<B: ByteOrderExt, const ALIGN: usize>(
                buffer: &impl Buf,
                offset: usize,
            ) -> Result<Self, CodecError> {
                let inner = FixedBytes::<{ Self::DATA_SIZE }>::decode::<B, ALIGN>(buffer, offset)?;
                Ok(Self(inner))
            }

            fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
                _buffer: &impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                Ok((offset, Self::DATA_SIZE))
            }
        }
    };
}

impl_evm_fixed!(Address);

impl<const BITS: usize, const LIMBS: usize> Encoder for Uint<BITS, LIMBS> {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = Self::BYTES;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buffer: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        if buffer.len() < aligned_offset + Self::DATA_SIZE {
            buffer.resize(aligned_offset + Self::DATA_SIZE, 0);
        }
        let bytes = &mut buffer[aligned_offset..aligned_offset + Self::DATA_SIZE];
        if B::is_big_endian() {
            bytes.copy_from_slice(&self.to_be_bytes_vec());
        } else {
            bytes.copy_from_slice(&self.to_le_bytes_vec());
        }

        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buffer: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        if buffer.remaining() < aligned_offset + Self::DATA_SIZE {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + Self::DATA_SIZE,
                found: buffer.remaining(),
                msg: "buffer too small to read Uint".to_string(),
            }));
        }

        let chunk = &buffer.chunk()[aligned_offset..aligned_offset + Self::DATA_SIZE];
        let value = if B::is_big_endian() {
            Self::from_be_slice(chunk)
        } else {
            Self::from_le_slice(chunk)
        };

        Ok(value)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        _buffer: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        Ok((aligned_offset, Self::DATA_SIZE))
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, LittleEndian};

    use super::*;
    #[cfg(test)]
    use alloy_primitives::{Address, U256};
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_write_to_existing_buf() {
        let existing_data = &[
            0, 0, 0, 0, 0, 0, 0, 32, // offset of the 1st bytes
            0, 0, 0, 0, 0, 0, 0, 12, // length of the 1st bytes
            0, 0, 0, 0, 0, 0, 0, 0, //
            0, 0, 0, 0, 0, 0, 0, 0, //
            72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, // b"Hello, World"
        ];
        let mut buf = BytesMut::new();
        buf.extend_from_slice(existing_data);

        let original = Bytes::from_static(b"Hello, World");
        // Write the data to the buffer
        let _result = write_bytes::<BigEndian, 8>(&mut buf, 16, &original);

        let expected = [
            0, 0, 0, 0, 0, 0, 0, 32, // offset of the 1st bytes
            0, 0, 0, 0, 0, 0, 0, 12, // length of the 1st bytes
            0, 0, 0, 0, 0, 0, 0, 44, // offset of the 2nd bytes
            0, 0, 0, 0, 0, 0, 0, 12, // length of the 2nd bytes
            72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, // b"Hello, World"
            72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, // b"Hello, World"
        ];

        assert_eq!(buf.to_vec(), expected);

        let mut encoded = buf.freeze();

        let decoded = read_bytes::<BigEndian, 8>(&mut encoded, 0);

        assert_eq!(decoded, original);
    }
    #[test]
    fn test_address_encode_decode() {
        let original = Address::from([0x42; 20]);
        let mut buffer = BytesMut::new();

        original.encode::<LittleEndian, 1>(&mut buffer, 0).unwrap();

        let encoded = buffer.freeze();
        println!("Encoded Address: {}", hex::encode(&encoded));

        let decoded = Address::decode::<LittleEndian, 1>(&mut encoded.clone(), 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_address_encode_decode_aligned() {
        let original = Address::from([0x42; 20]);
        let mut buffer = BytesMut::new();

        original.encode::<BigEndian, 8>(&mut buffer, 3).unwrap();

        let encoded = buffer.freeze();
        println!("Encoded Address (Aligned): {}", hex::encode(&encoded));

        let decoded = Address::decode::<BigEndian, 8>(&mut encoded.clone(), 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_le() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buffer = BytesMut::new();

        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        let encoded = buffer.freeze();
        println!("Encoded U256 (LE): {}", hex::encode(&encoded));
        let expected_encoded = "efcdab9078563412000000000000000000000000000000000000000000000000";
        assert_eq!(hex::encode(&encoded), expected_encoded);
        let decoded = U256::decode::<LittleEndian, 4>(&mut encoded.clone(), 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_be() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buffer = BytesMut::new();

        original.encode::<BigEndian, 4>(&mut buffer, 0).unwrap();

        let mut encoded = buffer.freeze();
        println!("Encoded U256 (BE): {}", hex::encode(&encoded));
        let expected_encoded = "0000000000000000000000000000000000000000000000001234567890abcdef";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = U256::decode::<BigEndian, 4>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
}
