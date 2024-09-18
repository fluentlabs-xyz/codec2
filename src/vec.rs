extern crate alloc;
use alloc::vec::Vec;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{
    encoder::{
        align_up, ByteOrderExt, CodecError, DecodingError, Encoder, read_u32_aligned,
        write_u32_aligned,
    },
    evm::{read_bytes, read_bytes_header, write_bytes},
};

/// We encode dynamic arrays as following:
/// - header
/// - + length - number of elements inside vector
/// - + offset - offset inside structure
/// - + size - number of encoded bytes
/// - body
/// - + raw bytes of the vector
///
/// TODO: clearify this: Why? And really?
/// We don't encode empty vectors, instead we store 0 as length,
/// it helps to reduce empty vector size from 12 to 4 bytes.
impl<T: Default + Sized + Encoder + std::fmt::Debug> Encoder for Vec<T> {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;
    const DATA_SIZE: usize = 0; // Dynamic size

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        // Check if we can store header
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_elem_size * 3, 0)
        };
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, self.len() as u32);

        // If vector is empty, we don't need to encode anything
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
            obj.encode::<B, ALIGN>(&mut value_encoder, elem_offset)
                .expect("Failed to encode vector element");
        }

        write_bytes::<B, ALIGN>(buf, aligned_offset + 4, &value_encoder.freeze());

        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_el_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + aligned_header_el_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_el_size,
                found: buf.remaining(),
                msg: "failed to decode vector length".to_string(),
            }));
        }

        let data_len = read_u32_aligned::<B, ALIGN>(buf, aligned_offset) as usize;
        if data_len == 0 {
            return Ok(Vec::new());
        }

        let input_bytes =
            read_bytes::<B, ALIGN>(buf, aligned_offset + aligned_header_el_size).unwrap();

        let mut result = Vec::with_capacity(data_len);

        // Aligned value size
        let val_size = align_up::<ALIGN>(T::HEADER_SIZE + T::DATA_SIZE);

        for i in 0..data_len {
            let elem_offset = i * val_size;
            // clone - copy only pointer to the buffer
            // we can't pass whole buffer, because some decoders (primitive types f.e.) can consume it
            let mut input_bytes = input_bytes.clone();
            let value = T::decode::<B, ALIGN>(&mut input_bytes, elem_offset)?;

            result.push(value);
        }

        Ok(result)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let elem_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + elem_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + elem_size,
                found: buf.remaining(),
                msg: "failed to decode vector length".to_string(),
            }));
        }

        let vec_len = read_u32_aligned::<B, ALIGN>(buf, aligned_offset) as usize;

        let (data_offset, data_length) = if vec_len == 0 {
            (0, 0)
        } else {
            read_bytes_header::<B, ALIGN>(buf, aligned_offset + elem_size).unwrap()
        };

        Ok((data_offset, data_length))
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, LittleEndian};

    use crate::encoder::Encoder;

    use super::*;

    #[test]
    fn test_empty_vec_u32() {
        let original: Vec<u32> = Vec::new();
        let mut buffer = BytesMut::new();

        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();
        let mut encoded = buffer.freeze();
        let expected = hex::decode("000000000c00000000000000").expect("Failed to decode hex");
        assert_eq!(encoded, Bytes::from(expected));

        let decoded = Vec::<u32>::decode::<LittleEndian, 4>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32() {
        let original: Vec<u32> = vec![1, 2, 3, 4];
        let mut buffer = BytesMut::new();

        original.encode::<BigEndian, 4>(&mut buffer, 0).unwrap();
        let mut encoded = buffer.freeze();

        let expected_encoded = "000000040000000c0000001000000001000000020000000300000004";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        println!("{:?}", hex::encode(&encoded));

        let decoded = <Vec<u32>>::decode::<BigEndian, 4>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32_with_offset() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        original.encode::<LittleEndian, 4>(&mut buffer, 3).unwrap();
        let mut encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));

        let decoded = Vec::<u32>::decode::<LittleEndian, 4>(&mut encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_vec_u8_with_offset() {
        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        original.encode::<LittleEndian, 4>(&mut buffer, 3).unwrap();
        let mut encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));

        let decoded: Vec<u8> = Vec::<u8>::decode::<LittleEndian, 4>(&mut encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 2>(&mut buffer, 0).unwrap();
        let mut encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));
        let expected_encoded = "020000000c00000022000000020000001800000004000000030000001c0000000600000003000400050006000700";

        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = Vec::<Vec<u16>>::decode::<LittleEndian, 2>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_nested_vec_a4_le() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();
        let mut encoded = buffer.freeze();
        let decoded = Vec::<Vec<u16>>::decode::<LittleEndian, 4>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_nested_vec_a4_be() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buffer = BytesMut::new();
        original.encode::<BigEndian, 4>(&mut buffer, 0).unwrap();
        let mut encoded = buffer.freeze();

        let decoded = Vec::<Vec<u16>>::decode::<BigEndian, 4>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_large_vec() {
        let original: Vec<u64> = (0..1000).collect();
        let mut buffer = BytesMut::new();

        original.encode::<BigEndian, 8>(&mut buffer, 0).unwrap();
        let mut encoded = buffer.freeze();

        let decoded = Vec::<u64>::decode::<BigEndian, 8>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
}
