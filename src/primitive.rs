extern crate alloc;

use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

use crate::encoder::{align_up, get_aligned_slice, is_big_endian, Encoder};
use crate::error::{CodecError, DecodingError};

impl<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool> Encoder<B, ALIGN, SOL_MODE> for u8 {
    const HEADER_SIZE: usize = core::mem::size_of::<u8>();

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size =
            align_up::<ALIGN>(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));

        if buf.len() < aligned_offset + word_size {
            buf.resize(aligned_offset + word_size, 0);
        }

        let write_to = get_aligned_slice::<B, ALIGN>(buf, aligned_offset, 1);

        write_to[0] = *self;

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

impl<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool> Encoder<B, ALIGN, SOL_MODE> for bool {
    const HEADER_SIZE: usize = core::mem::size_of::<bool>();

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let value: u8 = if *self { 1 } else { 0 };

        <u8 as Encoder<B, { ALIGN }, { SOL_MODE }>>::encode(&value, buf, offset)
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let value = <u8 as Encoder<B, { ALIGN }, { SOL_MODE }>>::decode(buf, offset)?;

        Ok(value != 0)
    }

    fn partial_decode(_buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
    }
}

macro_rules! impl_int {
    ($typ:ty, $read_method:ident, $write_method:ident) => {
        impl<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool> Encoder<B, ALIGN, SOL_MODE>
            for $typ
        {
            const HEADER_SIZE: usize = core::mem::size_of::<$typ>();

            fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);

                let word_size = align_up::<ALIGN>(
                    ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE),
                );

                if buf.len() < aligned_offset + word_size {
                    buf.resize(aligned_offset + word_size, 0);
                }

                let mut write_to = get_aligned_slice::<B, ALIGN>(
                    buf,
                    aligned_offset,
                    <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE,
                );

                B::$write_method(&mut write_to, *self);

                Ok(())
            }

            fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);
                let word_size = align_up::<ALIGN>(
                    ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE),
                );

                if buf.remaining() < aligned_offset + ALIGN {
                    return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }));
                }

                let chunk = &buf.chunk()[aligned_offset..];
                let value = if is_big_endian::<B>() {
                    B::$read_method(
                        &chunk[word_size - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE
                            ..word_size],
                    )
                } else {
                    B::$read_method(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
                };

                Ok(value)
            }

            fn partial_decode(
                _buf: &impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
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
/// The encoded data is prefixed with a single byte that indicates whether the Option is Some or None. Single byte will be aligned to ALIGN.
impl<T, B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool> Encoder<B, { ALIGN }, { SOL_MODE }>
    for Option<T>
where
    T: Sized + Encoder<B, { ALIGN }, { SOL_MODE }> + Default,
{
    const HEADER_SIZE: usize = 1 + T::HEADER_SIZE;

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        let required_space = aligned_offset + ALIGN.max(Self::HEADER_SIZE);
        if buf.len() < required_space {
            buf.resize(required_space, 0);
        }
        // Get aligned slice for the option flag
        let flag_slice = get_aligned_slice::<B, ALIGN>(buf, aligned_offset, 1);
        flag_slice[0] = if self.is_some() { 1 } else { 0 };

        let inner_offset = aligned_offset + ALIGN;

        match self {
            Some(inner_value) => inner_value.encode(buf, inner_offset)?,
            None => {
                let default_value = T::default();
                default_value.encode(buf, inner_offset)?;
            }
        };

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_data_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        if buf.remaining() < aligned_offset + aligned_data_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_data_size,
                found: buf.remaining(),
                msg: "buf too small".to_string(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..];
        let option_flag = if is_big_endian::<B>() {
            chunk[aligned_data_size - 1]
        } else {
            chunk[0]
        };

        let chunk = &buf.chunk()[aligned_offset + ALIGN..];

        if option_flag != 0 {
            let inner_value = T::decode(&chunk, 0)?;
            Ok(Some(inner_value))
        } else {
            Ok(None)
        }
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
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
        let option_flag = if is_big_endian::<B>() {
            chunk[ALIGN - 1]
        } else {
            chunk[0]
        };

        let chunk = &buf.chunk()[aligned_offset + ALIGN..];

        if option_flag != 0 {
            let (_, inner_size) = T::partial_decode(&chunk, 0)?;
            Ok((aligned_offset, aligned_header_size + inner_size))
        } else {
            let aligned_data_size = align_up::<ALIGN>(T::HEADER_SIZE);
            Ok((aligned_offset, aligned_header_size + aligned_data_size))
        }
    }
}

impl<T, B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool, const N: usize>
    Encoder<B, { ALIGN }, { SOL_MODE }> for [T; N]
where
    T: Sized + Encoder<B, { ALIGN }, { SOL_MODE }> + Default + Copy,
{
    const HEADER_SIZE: usize = T::HEADER_SIZE * N;

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let item_size = align_up::<ALIGN>(T::HEADER_SIZE);
        let total_size = aligned_offset + item_size * N;

        if buf.len() < total_size {
            buf.resize(total_size, 0);
        }

        for (i, item) in self.iter().enumerate() {
            item.encode(buf, aligned_offset + i * item_size)?;
        }

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let item_size = align_up::<ALIGN>(T::HEADER_SIZE);
        let total_size = aligned_offset + item_size * N;

        if buf.remaining() < total_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: total_size,
                found: buf.remaining(),
                msg: "buf too small".to_string(),
            }));
        }

        let mut result = [T::default(); N];
        let elem_size = align_up::<ALIGN>(T::HEADER_SIZE + T::HEADER_SIZE);

        for (i, item) in result.iter_mut().enumerate() {
            *item = T::decode(buf, aligned_offset + i * elem_size)?;
        }

        Ok(result)
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let item_size = align_up::<ALIGN>(T::HEADER_SIZE);
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

        let mut buf = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buf.capacity());

        let encoding_result =
            <u8 as Encoder<BigEndian, { ALIGNMENT }, false>>::encode(&original, &mut buf, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0000000000000000000000000000000000000000000000000000000000000001";

        assert_eq!(hex::encode(&buf), expected_encoded);

        let mut buf_for_decode = buf.clone().freeze();
        let decoded =
            <u8 as Encoder<BigEndian, { ALIGNMENT }, false>>::decode(&buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
        println!("encoded: {:?}", buf);

        let partial_decoded = <u8 as Encoder<BigEndian, { ALIGNMENT }, false>>::partial_decode(
            &buf.clone().freeze(),
            0,
        )
        .unwrap();
        assert_eq!(partial_decoded, (0, 1));
    }

    #[test]
    fn test_u8_le_encode_decode() {
        let original: u8 = 1;
        const ALIGNMENT: usize = 32;
        let mut buf = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buf.capacity());

        let encoding_result =
            <u8 as Encoder<LittleEndian, { ALIGNMENT }, false>>::encode(&original, &mut buf, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0100000000000000000000000000000000000000000000000000000000000000";

        let encoded = buf.freeze();
        println!("Encoded: {:?}", encoded);
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded =
            <u8 as Encoder<LittleEndian, { ALIGNMENT }, false>>::decode(&encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);

        let partial_decoded =
            <u8 as Encoder<LittleEndian, { ALIGNMENT }, false>>::partial_decode(&encoded, 0)
                .unwrap();

        assert_eq!(partial_decoded, (0, 1));
    }

    #[test]
    fn test_bool_be_encode_decode() {
        let original: bool = true;
        const ALIGNMENT: usize = 32;

        let mut buf = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buf.capacity());

        let encoding_result =
            <bool as Encoder<BigEndian, { ALIGNMENT }, false>>::encode(&original, &mut buf, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0000000000000000000000000000000000000000000000000000000000000001";

        assert_eq!(hex::encode(&buf), expected_encoded);

        let buf_for_decode = buf.clone().freeze();
        let decoded =
            <bool as Encoder<BigEndian, { ALIGNMENT }, false>>::decode(&buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
        println!("encoded: {:?}", buf);

        let partial_decoded = <bool as Encoder<BigEndian, { ALIGNMENT }, false>>::partial_decode(
            &buf.clone().freeze(),
            0,
        )
        .unwrap();
        assert_eq!(partial_decoded, (0, 1));
    }

    #[test]
    fn test_bool_le_encode_decode() {
        let original: bool = true;
        const ALIGNMENT: usize = 32;

        let mut buf = BytesMut::zeroed(ALIGNMENT);

        println!("Buffer capacity: {}", buf.capacity());

        let encoding_result =
            <bool as Encoder<LittleEndian, { ALIGNMENT }, false>>::encode(&original, &mut buf, 0);

        assert!(encoding_result.is_ok());

        let expected_encoded = "0100000000000000000000000000000000000000000000000000000000000000";

        assert_eq!(hex::encode(&buf), expected_encoded);

        let buf_for_decode = buf.clone().freeze();
        let decoded =
            <bool as Encoder<LittleEndian, { ALIGNMENT }, false>>::decode(&buf_for_decode, 0)
                .unwrap();

        assert_eq!(original, decoded);
        println!("encoded: {:?}", buf);

        let partial_decoded =
            <bool as Encoder<LittleEndian, { ALIGNMENT }, false>>::partial_decode(
                &buf.clone().freeze(),
                0,
            )
            .unwrap();
        assert_eq!(partial_decoded, (0, 1));
    }

    #[test]
    fn test_u32_encode_decode_le() {
        let original: u32 = 0x12345678;
        let mut buf = BytesMut::new();

        <u32 as Encoder<LittleEndian, 8, false>>::encode(&original, &mut buf, 0).unwrap();

        println!("Encoded: {:?}", buf);

        assert_eq!(buf.to_vec(), vec![0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0]);

        let buf_for_decode = buf.freeze();
        let decoded = <u32 as Encoder<LittleEndian, 8, false>>::decode(&buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_encode_decode_be() {
        let original: u32 = 0x12345678;
        let mut buf = BytesMut::new();

        <u32 as Encoder<BigEndian, 8, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("{:?}", hex::encode(&encoded));
        assert_eq!(
            &encoded,
            &vec![0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78]
        );

        let decoded = <u32 as Encoder<BigEndian, 8, false>>::decode(&encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_i64_encode_decode_be() {
        let original: i64 = 0x1234567890ABCDEF;
        let mut buf = BytesMut::new();

        <i64 as Encoder<BigEndian, 8, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("Encoded: {:?}", hex::encode(&encoded));
        assert_eq!(
            &encoded,
            &vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]
        );

        let decoded = <i64 as Encoder<BigEndian, 8, false>>::decode(&encoded, 0).unwrap();
        println!("Decoded: {}", decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_u32_wasm_abi_encode_decode() {
        let original: u32 = 0x12345678;
        let mut buf = BytesMut::new();

        // Encode
        <u32 as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        // Check encoded format
        assert_eq!(buf.to_vec(), vec![0x78, 0x56, 0x34, 0x12]);

        // Decode
        let decoded = <u32 as Encoder<LittleEndian, 4, false>>::decode(&buf, 0).unwrap();

        // Check decoded value
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_solidity_abi_encode_decode() {
        let original: u32 = 0x12345678;
        let mut buf = BytesMut::new();

        // Encode
        <u32 as Encoder<BigEndian, 32, true>>::encode(&original, &mut buf, 0).unwrap();

        // Check encoded format (32 bytes, right-aligned)
        let expected = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0x12, 0x34, 0x56, 0x78,
        ];
        assert_eq!(buf.to_vec(), expected);

        // Decode
        let decoded = <u32 as Encoder<BigEndian, 32, true>>::decode(&buf, 0).unwrap();

        // Check decoded value
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_u32_encode_decode() {
        let original: Option<u32> = Some(0x12345678);
        let mut buf = BytesMut::with_capacity(8);

        let ok = <Option<u32> as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0);
        assert!(ok.is_ok());

        let mut encoded = buf.freeze();
        println!("Encoded: {:?}", &encoded.to_vec());
        assert_eq!(
            encoded,
            Bytes::from_static(&[0x01, 0x00, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12])
        );

        let decoded = <Option<u32> as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0);

        assert_eq!(original, decoded.unwrap());
    }

    #[test]
    fn test_u8_array_encode_decode_le_with_alignment() {
        let original: [u8; 5] = [1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();

        <[u8; 5] as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        let mut encoded = buf.freeze();
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
        let decoded = <[u8; 5] as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();
        println!("Decoded: {:?}", decoded);

        assert_eq!(original, decoded);
    }
}
