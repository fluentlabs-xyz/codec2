use crate::{
    encoder::{Alignment, Encoder, Endian},
    evm::{read_bytes_header, write_bytes},
};
use bytes::{Bytes, BytesMut};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EmptyVec;

impl Encoder<EmptyVec> for EmptyVec {
    const HEADER_SIZE: usize = 12;

    fn encode<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = A::align(offset);

        if buffer.len() < aligned_offset + Self::HEADER_SIZE {
            buffer.resize(aligned_offset + Self::HEADER_SIZE, 0);
        };
        E::write::<u32>(&mut buffer[aligned_offset..aligned_offset + 4], 0);

        write_bytes::<A, E>(buffer, aligned_offset + 4, &[]);
    }

    fn decode_header<A: Alignment, E: Endian>(
        bytes: &Bytes,
        offset: usize,
        _result: &mut EmptyVec,
    ) -> (usize, usize) {
        let aligned_offset = A::align(offset);

        // TODO: d1r1 maybe we should return an error here?
        if bytes.len() < aligned_offset + 4 {
            return (0, 0);
        }

        let count = E::read::<u32>(&bytes[aligned_offset..aligned_offset + 4]) as usize;
        debug_assert_eq!(count, 0);
        read_bytes_header::<A, E>(bytes, aligned_offset + 4)
    }
}

#[cfg(test)]
mod tests {
    use crate::encoder::{Align0, LittleEndian};

    use super::*;

    #[test]
    fn test_empty() {
        let values = EmptyVec;

        let mut buffer = BytesMut::new();
        values.encode::<Align0, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("encoded = {:?}", hex::encode(&encoded));
        let expected = "000000000c00000000000000";
        assert_eq!(hex::encode(&encoded), expected);

        let mut decoded = Default::default();
        EmptyVec::decode_body::<Align0, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(values, decoded);
    }

    #[test]
    fn test_empty_with_offset() {
        let values = EmptyVec;
        let mut buffer = BytesMut::from(&[0xFF, 0xFF, 0xFF][..]);
        values.encode::<Align0, LittleEndian>(&mut buffer, 3);

        let expected = "ffffff000000000f00000000000000";
        let encoded = buffer.freeze();
        println!("encoded = {:?}", hex::encode(&encoded));
        assert_eq!(hex::encode(&encoded), expected);

        let mut decoded = Default::default();
        let (offset, length) =
            EmptyVec::decode_header::<Align0, LittleEndian>(&encoded, 3, &mut decoded);
        assert_eq!(offset, 15);
        assert_eq!(length, 0);

        let mut decoded = Default::default();

        EmptyVec::decode_body::<Align0, LittleEndian>(&encoded, 3, &mut decoded);
        assert_eq!(values, decoded);
    }
}
