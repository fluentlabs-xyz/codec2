use crate::encoder::Encoder;
use crate::encoder::{align_up, read_u32_aligned, write_u32_aligned};
use crate::error::{CodecError, DecodingError};
use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EmptyVec;

// Implementation for WASM mode (SOL_MODE = false)
impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for EmptyVec {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3; // 12 bytes

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        // Write number of elements (0 for EmptyVec)
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 0);

        // Write offset and length (both 0 for EmptyVec)
        write_u32_aligned::<B, ALIGN>(
            buf,
            aligned_offset + aligned_elem_size,
            (aligned_elem_size * 3) as u32,
        );
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size * 2, 0);

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE,
                found: buf.remaining(),
                msg: "failed to decode EmptyVec".to_string(),
            }));
        }

        let count = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?;
        if count != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "EmptyVec must have count of 0".to_string(),
            )));
        }

        // Read and verify offset and length
        let data_offset =
            read_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size)? as usize;
        let data_length =
            read_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size * 2)? as usize;

        if data_offset != <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE || data_length != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "Invalid offset or length for EmptyVec".to_string(),
            )));
        }

        Ok(EmptyVec)
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE,
                found: buf.remaining(),
                msg: "failed to partially decode EmptyVec".to_string(),
            }));
        }

        let count = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?;
        if count != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "EmptyVec must have count of 0".to_string(),
            )));
        }

        let data_offset =
            read_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size)? as usize;
        let data_length =
            read_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size * 2)? as usize;

        Ok((data_offset, data_length))
    }
}

// Implementation for Solidity mode (SOL_MODE = true)
impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for EmptyVec {
    const HEADER_SIZE: usize = 32; // Solidity uses 32 bytes for dynamic array header

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        // Write offset to data
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, (aligned_offset + 32) as u32);

        // Write length (0 for EmptyVec)
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset + 32, 0);

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        if buf.remaining() < aligned_offset + 32 {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + 32,
                found: buf.remaining(),
                msg: "failed to decode EmptyVec".to_string(),
            }));
        }

        let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
        let length = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;

        if length != 0 {
            return Err(CodecError::Decoding(DecodingError::InvalidData(
                "EmptyVec must have length of 0".to_string(),
            )));
        }

        Ok(EmptyVec)
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        if buf.remaining() < aligned_offset + 32 {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + 32,
                found: buf.remaining(),
                msg: "failed to partially decode EmptyVec".to_string(),
            }));
        }

        let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
        let length = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;

        Ok((data_offset, length))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{BigEndian, LittleEndian};

    #[test]
    fn test_empty_vec_wasm_little_endian() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::new();
        <EmptyVec as Encoder<LittleEndian, 4, false>>::encode(&empty_vec, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "000000000c00000000000000");

        let decoded = <EmptyVec as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            <EmptyVec as Encoder<LittleEndian, 4, false>>::partial_decode(&encoded, 0).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(length, 0);
    }

    #[test]
    fn test_empty_vec_wasm_big_endian() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::new();
        <EmptyVec as Encoder<BigEndian, 4, false>>::encode(&empty_vec, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "000000000000000c00000000");

        let decoded = <EmptyVec as Encoder<BigEndian, 4, false>>::decode(&encoded, 0).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            <EmptyVec as Encoder<BigEndian, 4, false>>::partial_decode(&encoded, 0).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(length, 0);
    }

    #[test]
    fn test_empty_vec_solidity() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::new();
        <EmptyVec as Encoder<BigEndian, 32, true>>::encode(&empty_vec, &mut buf, 0).unwrap();

        let encoded = buf.freeze();

        assert_eq!(hex::encode(&encoded), "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000");

        let decoded = <EmptyVec as Encoder<BigEndian, 32, true>>::decode(&encoded, 0).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            <EmptyVec as Encoder<BigEndian, 32, true>>::partial_decode(&encoded, 0).unwrap();
        assert_eq!(offset, 32);
        assert_eq!(length, 0);
    }

    #[test]
    fn test_empty_vec_wasm_with_offset() {
        let empty_vec = EmptyVec;
        let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF][..]);
        <EmptyVec as Encoder<LittleEndian, 4, false>>::encode(&empty_vec, &mut buf, 3).unwrap();

        let encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "ffffff00000000000c00000000000000");

        let decoded = <EmptyVec as Encoder<LittleEndian, 4, false>>::decode(&encoded, 3).unwrap();
        assert_eq!(empty_vec, decoded);

        let (offset, length) =
            <EmptyVec as Encoder<LittleEndian, 4, false>>::partial_decode(&encoded, 3).unwrap();
        assert_eq!(offset, 12);
        assert_eq!(length, 0);
    }
}
