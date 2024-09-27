extern crate alloc;
use alloc::vec::Vec;

use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

use crate::bytes::{read_bytes, read_bytes_wasm};
use crate::encoder::read_u32_aligned1;
use crate::error::{CodecError, DecodingError};
use crate::{
    bytes::{read_bytes_header, write_bytes_solidity, write_bytes_wasm},
    encoder::{align_up, read_u32_aligned, write_u32_aligned, Encoder},
};

/// We encode dynamic arrays as following:
/// - header
/// - + length - number of elements inside vector
/// - + offset - offset inside structure
/// - + size - number of encoded bytes
/// - body
/// - + raw bytes of the vector
///
///
/// For solidity we don't have size.
/// - header
/// - + offset
/// - body
/// - + length
/// - + raw bytes of the vector
///
/// Implementation for non-Solidity mode
impl<T, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for Vec<T>
where
    T: Default + Sized + Encoder<B, { ALIGN }, false> + std::fmt::Debug,
{
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);
        let aligned_header_size = aligned_elem_size * 3;

        // Check if we can store header
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }

        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, self.len() as u32);

        if self.is_empty() {
            write_u32_aligned::<B, ALIGN>(
                buf,
                aligned_offset + aligned_elem_size,
                aligned_header_size as u32,
            );
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size * 2, 0);

            return Ok(());
        }

        // Encode values
        let mut value_encoder = BytesMut::zeroed(ALIGN.max(T::HEADER_SIZE) * self.len());

        for (index, obj) in self.iter().enumerate() {
            let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;
            obj.encode(&mut value_encoder, elem_offset)?;
        }

        let data = value_encoder.freeze();

        write_bytes_wasm::<B, ALIGN>(buf, aligned_offset + aligned_elem_size, &data);

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_el_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + aligned_header_el_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_el_size,
                found: buf.remaining(),
                msg: "failed to decode vector length".to_string(),
            }));
        }

        let data_len = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
        if data_len == 0 {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(data_len);

        let data = read_bytes_wasm::<B, ALIGN>(buf, aligned_offset + aligned_header_el_size)?;

        // let val_size =
        // println!("val_size: {}", val_size);
        for i in 0..data_len {
            let elem_offset = i * align_up::<ALIGN>(T::HEADER_SIZE);
            let value = T::decode(&data, elem_offset)?;

            result.push(value);
        }

        Ok(result)
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        read_bytes_header::<B, ALIGN, false>(buf, offset)
    }
}
// Implementation forSolidity mode
impl<T, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for Vec<T>
where
    T: Default + Sized + Encoder<B, { ALIGN }, true> + std::fmt::Debug,
{
    const HEADER_SIZE: usize = 32;
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        // Check if we can store header
        if buf.len() < aligned_offset + 32 {
            buf.resize(aligned_offset + 32, 0);
        }

        // Write offset
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, buf.len() as u32);

        if self.is_empty() {
            write_u32_aligned::<B, ALIGN>(buf, buf.len(), 0);

            return Ok(());
        }

        // Encode values
        let mut value_encoder = BytesMut::zeroed(32 * self.len());

        for (index, obj) in self.iter().enumerate() {
            let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;
            obj.encode(&mut value_encoder, elem_offset)?;
        }

        let data = value_encoder.freeze();

        write_bytes_solidity::<B, ALIGN>(buf, aligned_offset + 32, &data, self.len() as u32);

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        // data_len - number of elements in the vector
        let (data_offset, data_len) = Self::partial_decode(buf, aligned_offset)?;

        if data_len == 0 {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(data_len);

        for i in 0..data_len {
            let elem_offset = i * align_up::<ALIGN>(T::HEADER_SIZE);

            let value = T::decode(&&buf.chunk()[data_offset..], elem_offset)?;
            result.push(value);
        }

        Ok(result)
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        read_bytes_header::<B, ALIGN, true>(buf, offset)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BigEndian, LittleEndian};
    use bytes::{Bytes, BytesMut};

    #[test]
    fn test_empty_vec_u32() {
        let original: Vec<u32> = Vec::new();
        let mut buf = BytesMut::new();

        <Vec<u32> as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();
        let expected = hex::decode("000000000c00000000000000").expect("Failed to decode hex");
        assert_eq!(encoded, Bytes::from(expected));

        let decoded = <Vec<u32> as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32_simple() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();

        <Vec<u32> as Encoder<BigEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();

        let expected_encoded = "000000050000000c000000140000000100000002000000030000000400000005";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = <Vec<u32> as Encoder<BigEndian, 4, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32_with_offset() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        <Vec<u32> as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 3).unwrap();
        let encoded = buf.freeze();

        let decoded = <Vec<u32> as Encoder<LittleEndian, 4, false>>::decode(&encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u8_with_offset() {
        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        <Vec<u8> as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 3).unwrap();
        let encoded = buf.freeze();

        let decoded = <Vec<u8> as Encoder<LittleEndian, 4, false>>::decode(&encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec_le_a2() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        <Vec<Vec<u16>> as Encoder<LittleEndian, 2, false>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();

        let expected_encoded = "020000000c00000022000000020000001800000004000000030000001c0000000600000003000400050006000700";

        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded =
            <Vec<Vec<u16>> as Encoder<LittleEndian, 2, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec_a4_le() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        <Vec<Vec<u16>> as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();
        let decoded =
            <Vec<Vec<u16>> as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec_a4_be() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        <Vec<Vec<u16>> as Encoder<BigEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();

        let decoded = <Vec<Vec<u16>> as Encoder<BigEndian, 4, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_large_vec() {
        let original: Vec<u64> = (0..1000).collect();
        let mut buf = BytesMut::new();

        <Vec<u64> as Encoder<BigEndian, 8, false>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();

        let decoded = <Vec<u64> as Encoder<BigEndian, 8, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    // New test for Solidity mode
    #[test]
    fn test_vec_u32_solidity_mode() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();

        <Vec<u32> as Encoder<BigEndian, 32, true>>::encode(&original, &mut buf, 0).unwrap();
        let encoded = buf.freeze();

        let decoded = <Vec<u32> as Encoder<BigEndian, 32, true>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
}
