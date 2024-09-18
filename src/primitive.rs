extern crate alloc;

use bytes::{Buf, BytesMut};

use crate::encoder::{align, align_up, ByteOrderExt, Encoder};

use crate::error::{CodecError, DecodingError};

impl Encoder for u8 {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = core::mem::size_of::<u8>();

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        // Align the offset and header size
        let aligned_offset = align_up::<ALIGN>(offset);

        let word_size = align_up::<ALIGN>(ALIGN.max(Self::DATA_SIZE));

        if buf.len() < aligned_offset + word_size {
            // Resize the buffer to fit the encoded data
            buf.resize(aligned_offset + word_size, 0);
        }

        let aligned_value = align::<B, ALIGN>(&[*self]);
        buf[aligned_offset..aligned_offset + word_size].copy_from_slice(&aligned_value);
        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size = align_up::<ALIGN>(ALIGN.max(Self::DATA_SIZE));

        if buf.remaining() < aligned_offset + word_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + word_size,
                found: buf.remaining(),
                msg: "buf too small to read aligned u8".to_string(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..];
        let value = if B::is_big_endian() {
            chunk[word_size - 1]
        } else {
            chunk[0]
        };

        Ok(value)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        _buf: &impl Buf,
        _offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        Ok((0, Self::DATA_SIZE))
    }
}

impl Encoder for bool {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = core::mem::size_of::<bool>();

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let value: u8 = if *self { 1 } else { 0 };

        value.encode::<B, ALIGN>(buf, offset)
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let value = u8::decode::<B, ALIGN>(buf, offset)?;

        Ok(value != 0)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        _buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        Ok((offset, Self::DATA_SIZE))
    }
}

macro_rules! impl_int {
    ($typ:ty, $read_method:ident, $write_method:ident) => {
        impl Encoder for $typ {
            const HEADER_SIZE: usize = 0;
            const DATA_SIZE: usize = core::mem::size_of::<$typ>();

            fn encode<B: ByteOrderExt, const ALIGN: usize>(
                &self,
                buf: &mut BytesMut,
                offset: usize,
            ) -> Result<(), CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);

                let word_size = align_up::<ALIGN>(ALIGN.max(Self::DATA_SIZE));

                if buf.len() < aligned_offset + word_size {
                    buf.resize(aligned_offset + word_size, 0);
                }

                let mut bytes = [0u8; Self::DATA_SIZE];
                B::$write_method(&mut bytes, *self);

                let aligned_value = align::<B, ALIGN>(&bytes);
                buf[aligned_offset..aligned_offset + word_size].copy_from_slice(&aligned_value);
                Ok(())
            }

            fn decode<B: ByteOrderExt, const ALIGN: usize>(
                buf: &impl Buf,
                offset: usize,
            ) -> Result<Self, CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);
                let word_size = align_up::<ALIGN>(ALIGN.max(Self::DATA_SIZE));

                if buf.remaining() < aligned_offset + ALIGN {
                    return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small".to_string(),
                    }));
                }

                let chunk = &buf.chunk()[aligned_offset..];
                let value = if B::is_big_endian() {
                    B::$read_method(&chunk[word_size - Self::DATA_SIZE..word_size])
                } else {
                    B::$read_method(&chunk[..Self::DATA_SIZE])
                };
                // buf.advance(aligned_offset + word_size);

                Ok(value)
            }

            fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
                _buf: &impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                Ok((offset, Self::DATA_SIZE))
            }
        }
    };
}

impl_int!(u16, read_u16, write_u16);
impl_int!(u32, read_u32, write_u32);
impl_int!(u64, read_u64, write_u64);
impl_int!(i16, read_i16, write_i16);
impl_int!(i32, read_i32, write_i32);
impl_int!(i64, read_i64, write_i64);

/// Encodes and decodes Option<T> where T is an Encoder.
/// The encoded data is prefixed with a single byte that indicates whether the Option is Some or None. Single byte will be aligned to ALIGN. So
impl<T: Sized + Encoder + Default> Encoder for Option<T> {
    const HEADER_SIZE: usize = 1 + T::HEADER_SIZE;
    const DATA_SIZE: usize = T::DATA_SIZE;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
        let aligned_data_size = align_up::<ALIGN>(T::DATA_SIZE);

        let required_space = aligned_offset + aligned_header_size + aligned_data_size;

        if buf.len() < required_space {
            buf.resize(required_space, 0);
        }

        let option_flag: u8 = if self.is_some() { 1 } else { 0 };

        let aligned_option_flag = align::<B, ALIGN>(&[option_flag]);

        buf[aligned_offset..aligned_offset + aligned_option_flag.len()]
            .copy_from_slice(&aligned_option_flag);

        if let Some(inner_value) = self {
            inner_value.encode::<B, ALIGN>(buf, aligned_offset + aligned_option_flag.len())?;
        } else {
            let default_value = T::default();
            default_value.encode::<B, ALIGN>(buf, aligned_offset + aligned_option_flag.len())?;
        };
        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
        let aligned_data_size = align_up::<ALIGN>(T::DATA_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size + aligned_data_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_size + aligned_data_size,
                msg: "buf too small".to_string(),
                found: buf.remaining(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..];
        let option_flag = if B::is_big_endian() {
            //
            chunk[aligned_data_size - 1]
        } else {
            chunk[0]
        };

        let chunk = &buf.chunk()[aligned_offset + ALIGN..];

        if option_flag != 0 {
            let inner_value = T::decode::<B, ALIGN>(&chunk, 0)?;
            Ok(Some(inner_value))
        } else {
            Ok(None)
        }
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_size,
                found: buf.remaining(),
                msg: "buf too small".to_string(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..];
        let option_flag = if B::is_big_endian() {
            chunk[ALIGN - 1]
        } else {
            chunk[0]
        };

        let chunk = &buf.chunk()[aligned_offset + ALIGN..];

        if option_flag != 0 {
            let (_, inner_size) = T::partial_decode::<B, ALIGN>(&chunk, 0)?;
            Ok((aligned_offset, aligned_header_size + inner_size))
        } else {
            let aligned_data_size = align_up::<ALIGN>(T::DATA_SIZE);
            Ok((aligned_offset, aligned_header_size + aligned_data_size))
        }
    }
}

impl<T: Sized + Encoder + Default + Copy, const N: usize> Encoder for [T; N] {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = T::DATA_SIZE * N;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let item_size = align_up::<ALIGN>(T::DATA_SIZE);
        let total_size = aligned_offset + item_size * N;

        if buf.len() < total_size {
            buf.resize(total_size, 0);
        }

        for (i, item) in self.iter().enumerate() {
            item.encode::<B, ALIGN>(buf, aligned_offset + i * item_size)?;
        }

        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let item_size = align_up::<ALIGN>(T::DATA_SIZE);
        let total_size = aligned_offset + item_size * N;

        if buf.remaining() < total_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: total_size,
                found: buf.remaining(),
                msg: "buf too small".to_string(),
            }));
        }

        let mut result = [T::default(); N];
        let elem_size = align_up::<ALIGN>(T::HEADER_SIZE + T::DATA_SIZE);

        for (i, item) in result.iter_mut().enumerate() {
            // Offset is always 0 - we are advancing the buffer by reading the item
            *item = T::decode::<B, ALIGN>(buf, i * elem_size)?;
        }

        Ok(result)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let item_size = align_up::<ALIGN>(T::DATA_SIZE);
        let total_size = item_size * N;

        if buf.remaining() < aligned_offset + total_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + total_size,
                found: buf.remaining(),
                msg: "Buffer too small to decode array".to_string(),
            }));
        }

        Ok((aligned_offset, total_size))
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, LittleEndian};
    use bytes::{Bytes, BytesMut};

    use super::*;

    #[test]
    fn test_u8_be_encode_decode() {
        let original: u8 = 1;
        const ALIGNMENT: usize = 32;

        let mut buffer = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buffer.capacity());

        let encoding_result = original.encode::<BigEndian, ALIGNMENT>(&mut buffer, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0000000000000000000000000000000000000000000000000000000000000001";

        assert_eq!(hex::encode(&buffer), expected_encoded);

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = u8::decode::<BigEndian, 32>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
        println!("encoded: {:?}", buffer);

        let partial_decoded =
            u8::partial_decode::<BigEndian, 32>(&mut buffer.clone().freeze(), 0).unwrap();
        assert_eq!(partial_decoded, (0, 1));
    }
    #[test]
    fn test_u8_le_encode_decode() {
        let original: u8 = 1;
        const ALIGNMENT: usize = 32;
        let mut buffer = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buffer.capacity());

        let encoding_result = original.encode::<LittleEndian, ALIGNMENT>(&mut buffer, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0100000000000000000000000000000000000000000000000000000000000000";

        let mut encoded = buffer.freeze();
        println!("Encoded: {:?}", encoded);
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = u8::decode::<LittleEndian, 32>(&mut encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);

        let partial_decoded = u8::partial_decode::<LittleEndian, 32>(&mut encoded, 0).unwrap();

        assert_eq!(partial_decoded, (0, 1));
    }
    #[test]
    fn test_bool_be_encode_decode() {
        let original: bool = true;
        const ALIGNMENT: usize = 32;

        let mut buffer = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buffer.capacity());

        let encoding_result = original.encode::<BigEndian, ALIGNMENT>(&mut buffer, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0000000000000000000000000000000000000000000000000000000000000001";

        assert_eq!(hex::encode(&buffer), expected_encoded);

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = bool::decode::<BigEndian, 32>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
        println!("encoded: {:?}", buffer);

        let partial_decoded =
            bool::partial_decode::<BigEndian, 32>(&mut buffer.clone().freeze(), 0).unwrap();
        assert_eq!(partial_decoded, (0, 1));
    }
    #[test]
    fn test_bool_le_encode_decode() {
        let original: bool = true;
        const ALIGNMENT: usize = 32;
        let mut buffer = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buffer.capacity());

        let encoding_result = original.encode::<LittleEndian, ALIGNMENT>(&mut buffer, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0100000000000000000000000000000000000000000000000000000000000000";

        let mut encoded = buffer.freeze();
        println!("Encoded: {:?}", encoded);
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = bool::decode::<LittleEndian, 32>(&mut encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);

        let partial_decoded = u8::partial_decode::<LittleEndian, 32>(&mut encoded, 0).unwrap();

        assert_eq!(partial_decoded, (0, 1));
    }

    #[test]
    fn test_u32_encode_decode_le() {
        let original: u32 = 0x12345678;
        let mut buffer = BytesMut::new();

        original.encode::<LittleEndian, 8>(&mut buffer, 0).unwrap();

        assert_eq!(buffer.to_vec(), vec![0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0]);

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = u32::decode::<LittleEndian, 4>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_encode_decode_be() {
        let original: u32 = 0x12345678;
        let mut buffer = BytesMut::new();

        original.encode::<BigEndian, 8>(&mut buffer, 0).unwrap();

        let mut encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));
        assert_eq!(
            &encoded,
            &vec![0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78]
        );

        let decoded = u32::decode::<BigEndian, 8>(&mut encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_i64_encode_decode_be() {
        let original: i64 = 0x1234567890ABCDEF;
        let mut buffer = BytesMut::new();

        original.encode::<BigEndian, 8>(&mut buffer, 0).unwrap();

        let mut encoded = buffer.freeze();
        println!("Encoded: {:?}", hex::encode(&encoded));
        assert_eq!(
            &encoded,
            &vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]
        );

        let decoded = i64::decode::<BigEndian, 8>(&mut encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_u32_encode_decode() {
        let original: Option<u32> = Some(0x12345678);
        let mut buffer = BytesMut::with_capacity(8);

        let ok = original.encode::<LittleEndian, 4>(&mut buffer, 0);
        assert!(ok.is_ok());

        let mut encoded = buffer.freeze();
        println!("Encoded: {:?}", &encoded.to_vec());
        assert_eq!(
            encoded,
            Bytes::from_static(&[0x01, 0x00, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12])
        );

        let decoded = Option::<u32>::decode::<LittleEndian, 4>(&mut encoded, 0);

        assert_eq!(original, decoded.unwrap());
    }

    #[test]
    fn test_u8_array_encode_decode_le_with_alignment() {
        let original: [u8; 5] = [1, 2, 3, 4, 5];
        let mut buffer = BytesMut::new();

        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        let mut encoded = buffer.freeze();
        println!("Encoded: {:?}", hex::encode(&encoded));

        // Check that the encoded data is correct and properly aligned
        assert_eq!(
            &encoded.to_vec(),
            &[
                0x01, 0x00, 0x00, 0x00, // First byte aligned to 4 bytes
                0x02, 0x00, 0x00, 0x00, // Second byte aligned to 4 bytes
                0x03, 0x00, 0x00, 0x00, // Third byte aligned to 4 bytes
                0x04, 0x00, 0x00, 0x00, // Fourth byte aligned to 4 bytes
                0x05, 0x00, 0x00, 0x00 // Fifth byte aligned to 4 bytes
            ]
        );

        println!("Encoded: {:?}", encoded.to_vec());
        println!("encoded len: {}", encoded.len());
        let decoded = <[u8; 5]>::decode::<LittleEndian, 4>(&mut encoded, 0).unwrap();
        println!("Decoded: {:?}", decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_u32_array_encode_decode_le_with_alignment() {
        let original: [u32; 5] = [1, 2, 3, 4, 5];
        let mut buffer = BytesMut::new();

        original.encode::<LittleEndian, 8>(&mut buffer, 0).unwrap();

        let mut encoded = buffer.freeze();
        println!("Encoded: {:?}", hex::encode(&encoded));

        // Check that the encoded data is correct and properly aligned
        assert_eq!(
            &encoded.to_vec(),
            &[
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, // First u32 aligned to 8 bytes
                0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, // Second u32 aligned to 8 bytes
                0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, // Third u32 aligned to 8 bytes
                0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, // Fourth u32 aligned to 8 bytes
                0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00 // Fifth u32 aligned to 8 bytes
            ]
        );

        println!("Encoded: {:?}", encoded.to_vec());
        println!("encoded len: {}", encoded.len());
        let decoded = <[u32; 5]>::decode::<LittleEndian, 8>(&mut encoded, 0).unwrap();
        println!("Decoded: {:?}", decoded);

        assert_eq!(original, decoded);
    }
}
