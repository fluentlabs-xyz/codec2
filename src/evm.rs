use std::io::Read;

use crate::encoder::{Alignment, Encoder, Endianness};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use bytes::BytesMut;

// Write bytes to the buffer
// Returns the size of the header
// To avoid resizing buffer, you can pre-allocate the buffer with the size of the header before calling this function
// The header contains the offset and length of the data
// The actual data is appended to the buffer, after the header
pub fn write_bytes<A: Alignment, E: Endianness>(
    buffer: &mut BytesMut,
    offset: usize,
    bytes: &[u8],
) -> usize {
    let aligned_offset = A::align(offset);

    // header = offset (u32) + data_length (u32)
    // u32 + u32 = 8 bytes
    let header_size = A::align(8);

    if buffer.len() < aligned_offset + header_size {
        buffer.resize(aligned_offset + header_size, 0);
    }
    // Offset of the data since the beginning of the buffer
    let data_offset = buffer.len();

    // Write header
    E::write_u32(
        &mut buffer[aligned_offset..aligned_offset + 4],
        data_offset as u32,
    );
    E::write_u32(
        &mut buffer[aligned_offset + 4..aligned_offset + 8],
        bytes.len() as u32,
    );

    // Append data
    buffer.extend_from_slice(bytes);

    header_size
}

pub fn read_bytes<A: Alignment, E: Endianness>(
    bytes: &bytes::Bytes,
    field_offset: usize,
) -> bytes::Bytes {
    let aligned_header_offset = A::align(field_offset);
    let slice = bytes.slice(aligned_header_offset..);
    let offset = E::read_u32(&slice[..4]) as usize;
    let length = E::read_u32(&slice[4..8]) as usize;
    bytes.slice(offset..offset + length)
}

pub fn read_bytes_header<A: Alignment, E: Endianness>(
    bytes: &bytes::Bytes,
    field_offset: usize,
) -> (usize, usize) {
    let aligned_header_offset = A::align(field_offset);
    let slice = bytes.slice(aligned_header_offset..);
    let offset = E::read_u32(&slice[..4]) as usize;
    let length = E::read_u32(&slice[4..8]) as usize;
    (offset, length)
}

impl Encoder<Bytes> for Bytes {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        write_bytes::<A, E>(buffer, field_offset, self);
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        _result: &mut Bytes,
    ) -> (usize, usize) {
        let aligned_header_offset = A::align(field_offset);
        let slice = bytes.slice(aligned_header_offset..);
        let offset = E::read_u32(&slice[..4]) as usize;
        let length = E::read_u32(&slice[4..8]) as usize;
        (offset, length)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        result: &mut Bytes,
    ) {
        let (offset, length) = Self::decode_header::<A, E>(bytes, field_offset, result);
        let data = bytes.slice(offset..offset + length);
        *result = Bytes::copy_from_slice(&data);
    }
}

impl<const N: usize> Encoder<FixedBytes<N>> for FixedBytes<N> {
    const HEADER_SIZE: usize = N;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        let aligned_offset = A::align(field_offset);
        buffer.resize(aligned_offset + N, 0);
        buffer[aligned_offset..aligned_offset + N].copy_from_slice(&self.0);
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        result: &mut FixedBytes<N>,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        result
            .0
            .copy_from_slice(&bytes[aligned_offset..aligned_offset + N]);
        (0, 0)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        result: &mut FixedBytes<N>,
    ) {
        Self::decode_header::<A, E>(bytes, field_offset, result);
    }
}

macro_rules! impl_evm_fixed {
    ($typ:ty) => {
        impl Encoder<$typ> for $typ {
            const HEADER_SIZE: usize = <$typ>::len_bytes();

            fn encode<A: Alignment, E: Endianness>(
                &self,
                buffer: &mut BytesMut,
                field_offset: usize,
            ) {
                self.0.encode::<A, E>(buffer, field_offset);
            }

            fn decode_header<A: Alignment, E: Endianness>(
                bytes: &bytes::Bytes,
                field_offset: usize,
                result: &mut $typ,
            ) -> (usize, usize) {
                FixedBytes::<{ Self::HEADER_SIZE }>::decode_header::<A, E>(
                    bytes,
                    field_offset,
                    &mut result.0,
                )
            }

            fn decode_body<A: Alignment, E: Endianness>(
                bytes: &bytes::Bytes,
                field_offset: usize,
                result: &mut $typ,
            ) {
                Self::decode_header::<A, E>(bytes, field_offset, result);
            }
        }
    };
}

impl_evm_fixed!(Address);

impl<const BITS: usize, const LIMBS: usize> Encoder<Uint<BITS, LIMBS>> for Uint<BITS, LIMBS> {
    const HEADER_SIZE: usize = Self::BYTES;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        let aligned_offset = A::align(field_offset);
        buffer.resize(aligned_offset + Self::BYTES, 0);
        let bytes = &mut buffer[aligned_offset..aligned_offset + Self::BYTES];
        if E::is_little_endian() {
            bytes.copy_from_slice(&self.as_le_bytes());
        } else {
            bytes.copy_from_slice(&self.to_be_bytes_vec());
        }
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        result: &mut Uint<BITS, LIMBS>,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        let slice = bytes.slice(aligned_offset..aligned_offset + Self::BYTES);

        if E::is_little_endian() {
            *result = Uint::from_le_slice(&slice[..Self::BYTES]);
        } else {
            *result = Uint::from_be_slice(&slice[..Self::BYTES]);
        }
        (0, 0)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        result: &mut Uint<BITS, LIMBS>,
    ) {
        Self::decode_header::<A, E>(bytes, field_offset, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{Align1, Align4, Align8, BigEndian, LittleEndian};
    use alloy_primitives::{Address, U256};
    // use hex;
    // use hex_literal::hex;

    #[test]
    fn test_address_encode_decode() {
        let original = Address::from([0x42; 20]);
        let mut buffer = BytesMut::new();

        original.encode::<Align1, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("Encoded Address: {}", hex::encode(&encoded));

        let mut decoded = Address::default();
        Address::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_address_encode_decode_aligned() {
        let original = Address::from([0x42; 20]);
        let mut buffer = BytesMut::new();

        original.encode::<Align8, BigEndian>(&mut buffer, 3);

        let encoded = buffer.freeze();
        println!("Encoded Address (Aligned): {}", hex::encode(&encoded));

        let mut decoded = Address::default();
        Address::decode_body::<Align8, BigEndian>(&encoded, 3, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_le() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buffer = BytesMut::new();

        original.encode::<Align4, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("Encoded U256 (LE): {}", hex::encode(&encoded));

        let mut decoded = U256::ZERO;
        U256::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_be() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buffer = BytesMut::new();

        original.encode::<Align8, BigEndian>(&mut buffer, 5);

        let encoded = buffer.freeze();
        println!("Encoded U256 (BE): {}", hex::encode(&encoded));

        let mut decoded = U256::ZERO;
        U256::decode_body::<Align8, BigEndian>(&encoded, 5, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_large_number() {
        let value =
            hex_literal::hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let original = U256::from_be_bytes(value);
        let mut buffer = BytesMut::new();

        original.encode::<Align1, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("Encoded large U256: {}", hex::encode(&encoded));

        let mut decoded = U256::ZERO;
        U256::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_mixed_endianness() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buffer = BytesMut::new();

        original.encode::<Align4, BigEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("Encoded U256 (BE): {}", hex::encode(&encoded));

        let mut decoded = U256::ZERO;
        U256::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

        // This should fail because we're encoding in BE and decoding in LE
        assert_ne!(original, decoded);

        // Correct decoding
        let mut correct_decoded = U256::ZERO;
        U256::decode_body::<Align4, BigEndian>(&encoded, 0, &mut correct_decoded);
        assert_eq!(original, correct_decoded);
    }

    #[test]
    fn test_bytes_a1() {
        let original = Bytes::from_static(b"Hello, World");
        let mut buffer = BytesMut::new();
        original.encode::<Align1, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("Encoded (A1): {}", hex::encode(&encoded));

        let mut decoded = Bytes::new();
        Bytes::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
        assert_eq!(
            hex::encode(&encoded),
            "080000000c00000048656c6c6f2c20576f726c64"
        );
    }

    #[test]
    fn test_bytes_a8() {
        let original = Bytes::from_static(b"Hello, World");
        let mut buffer = BytesMut::new();
        original.encode::<Align8, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("Encoded (A8): {}", hex::encode(&encoded));

        let mut decoded = Bytes::new();
        Bytes::decode_body::<Align8, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
        assert_eq!(
            hex::encode(&encoded),
            "080000000c00000048656c6c6f2c20576f726c64"
        );
    }

    #[test]
    fn test_bytes_a8_with_offset() {
        let original = Bytes::from_static(b"Hello");
        let mut buffer = BytesMut::from(&[0xFF, 0xFF, 0xFF][..]);
        original.encode::<Align8, LittleEndian>(&mut buffer, 3);

        let encoded = buffer.freeze();
        println!("Encoded (A8 with offset): {}", hex::encode(&encoded));

        let mut decoded = Bytes::new();
        Bytes::decode_body::<Align8, LittleEndian>(&encoded, 3, &mut decoded);

        assert_eq!(original, decoded);
        assert_eq!(
            hex::encode(&encoded),
            "ffffff0000000000100000000500000048656c6c6f"
        );
    }
}
