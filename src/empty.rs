use crate::encoder::Encoder;
use crate::encoder::{align_up, read_u32_aligned, write_u32_aligned};
use crate::error::{CodecError, DecodingError};
use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EmptyVec;

impl Encoder for EmptyVec {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3; // 12 bytes

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        // Write number of elements (0 for EmptyVec)
        write_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset, 0);

        // Write offset and length (both 0 for EmptyVec)
        write_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_elem_size,
            (aligned_elem_size * 3) as u32,
        );
        write_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_elem_size * 2,
            0,
        );

        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + Self::HEADER_SIZE {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + Self::HEADER_SIZE,
                found: buf.remaining(),
                msg: "failed to decode EmptyVec".to_string(),
            }));
        }

        let count = read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset)?;
        if count != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "EmptyVec must have count of 0".to_string(),
            )));
        }

        // Read and verify offset and length
        let data_offset =
            read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset + aligned_elem_size)?
                as usize;
        let data_length = read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_elem_size * 2,
        )? as usize;

        if data_offset != Self::HEADER_SIZE || data_length != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "Invalid offset or length for EmptyVec".to_string(),
            )));
        }

        Ok(EmptyVec)
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + Self::HEADER_SIZE {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + Self::HEADER_SIZE,
                found: buf.remaining(),
                msg: "failed to partially decode EmptyVec".to_string(),
            }));
        }

        let count = read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset)?;
        if count != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "EmptyVec must have count of 0".to_string(),
            )));
        }

        let data_offset =
            read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset + aligned_elem_size)?
                as usize;
        let data_length = read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_elem_size * 2,
        )? as usize;

        Ok((data_offset, data_length))
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, LittleEndian};

    use super::*;

    #[test]
    fn test_empty_vec_little_endian() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::new();
        empty_vec
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();

        let mut encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "000000000c00000000000000");

        let decoded = EmptyVec::decode::<LittleEndian, 4, false>(&mut encoded.clone(), 0).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            EmptyVec::partial_decode::<LittleEndian, 4, false>(&mut encoded, 0).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(length, 0);
    }

    #[test]
    fn test_empty_vec_big_endian() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::new();
        empty_vec
            .encode::<BigEndian, 4, false>(&mut buf, 0)
            .unwrap();

        let mut encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "000000000000000c00000000");

        let decoded = EmptyVec::decode::<BigEndian, 4, false>(&mut encoded.clone(), 0).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            EmptyVec::partial_decode::<BigEndian, 4, false>(&mut encoded, 0).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(length, 0);
    }

    #[test]
    fn test_empty_vec_with_offset() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF][..]);
        empty_vec
            .encode::<LittleEndian, 4, false>(&mut buf, 3)
            .unwrap();

        let mut encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "ffffff00000000000c00000000000000");

        let decoded = EmptyVec::decode::<LittleEndian, 4, false>(&mut encoded.clone(), 3).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            EmptyVec::partial_decode::<LittleEndian, 4, false>(&mut encoded, 3).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(length, 0);
    }
}
