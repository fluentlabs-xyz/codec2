use std::usize;

use crate::bytes::{read_bytes, read_bytes_header, write_bytes};
use crate::encoder::{align_up, is_big_endian, write_u32_aligned, Encoder};
use crate::error::{CodecError, DecodingError};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

impl Encoder for Bytes {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let elem_size = align_up::<ALIGN>(4);
        if buf.len() < aligned_offset + elem_size {
            buf.resize(aligned_offset + elem_size, 0);
        }
        let data_offset = buf.len();

        write_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset, data_offset as u32);
        let _ = write_bytes::<B, ALIGN, SOLIDITY_COMP>(buf, offset, self);
        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        Ok(Self::from(
            read_bytes::<B, ALIGN, SOLIDITY_COMP>(buf, offset).unwrap(),
        ))
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(buf, offset)
    }
}

impl<const N: usize> Encoder for FixedBytes<N> {
    const HEADER_SIZE: usize = N;

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        if buf.len() < offset + N {
            buf.resize(offset + N, 0);
        }
        buf[offset..offset + N].copy_from_slice(self.as_ref());
        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        let data = buf.chunk()[offset..offset + N].to_vec();
        Ok(FixedBytes::from_slice(&data))
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        _buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        Ok((offset, N))
    }
}

macro_rules! impl_evm_fixed {
    ($typ:ty) => {
        impl Encoder for $typ {
            const HEADER_SIZE: usize = <$typ>::len_bytes();

            fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
                &self,
                buf: &mut BytesMut,
                offset: usize,
            ) -> Result<(), CodecError> {
                self.0.encode::<B, ALIGN, SOLIDITY_COMP>(buf, offset)
            }

            fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
                buf: &(impl Buf + ?Sized),
                offset: usize,
            ) -> Result<Self, CodecError> {
                let inner = FixedBytes::<{ Self::HEADER_SIZE }>::decode::<B, ALIGN, SOLIDITY_COMP>(
                    buf, offset,
                )?;
                Ok(Self(inner))
            }

            fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
                _buf: &(impl Buf + ?Sized),
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                Ok((offset, Self::HEADER_SIZE))
            }
        }
    };
}

impl_evm_fixed!(Address);

impl<const BITS: usize, const LIMBS: usize> Encoder for Uint<BITS, LIMBS> {
    const HEADER_SIZE: usize = Self::BYTES;

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        if buf.len() < aligned_offset + Self::HEADER_SIZE {
            buf.resize(aligned_offset + Self::HEADER_SIZE, 0);
        }
        let bytes = &mut buf[aligned_offset..aligned_offset + Self::HEADER_SIZE];
        if is_big_endian::<B>() {
            bytes.copy_from_slice(&self.to_be_bytes_vec());
        } else {
            bytes.copy_from_slice(&self.to_le_bytes_vec());
        }

        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + Self::HEADER_SIZE {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + Self::HEADER_SIZE,
                found: buf.remaining(),
                msg: "buf too small to read Uint".to_string(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..aligned_offset + Self::HEADER_SIZE];
        let value = if is_big_endian::<B>() {
            Self::from_be_slice(chunk)
        } else {
            Self::from_le_slice(chunk)
        };

        Ok(value)
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        _buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        Ok((aligned_offset, Self::HEADER_SIZE))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use alloy_primitives::{Address, U256};
    use byteorder::{BigEndian, LittleEndian};
    use bytes::BytesMut;

    use super::*;

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
        // Write the data to the buf
        let _result = write_bytes::<BigEndian, 8, false>(&mut buf, 16, &original);

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

        let decoded = read_bytes::<BigEndian, 8, false>(&mut encoded, 0).unwrap();

        assert_eq!(decoded.to_vec(), original.to_vec());
    }
    #[test]
    fn test_address_encode_decode() {
        let original = Address::from([0x42; 20]);
        let mut buf = BytesMut::new();

        original
            .encode::<LittleEndian, 1, false>(&mut buf, 0)
            .unwrap();

        let encoded = buf.freeze();
        println!("Encoded Address: {}", hex::encode(&encoded));

        let decoded = Address::decode::<LittleEndian, 1, false>(&mut encoded.clone(), 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_address_encode_decode_aligned() {
        let original = Address::from([0x42; 20]);
        let mut buf = BytesMut::new();

        original.encode::<BigEndian, 8, false>(&mut buf, 3).unwrap();

        let encoded = buf.freeze();
        println!("Encoded Address (Aligned): {}", hex::encode(&encoded));

        let decoded = Address::decode::<BigEndian, 8, false>(&mut encoded.clone(), 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_le() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buf = BytesMut::new();

        original
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();

        let encoded = buf.freeze();
        println!("Encoded U256 (LE): {}", hex::encode(&encoded));
        let expected_encoded = "efcdab9078563412000000000000000000000000000000000000000000000000";
        assert_eq!(hex::encode(&encoded), expected_encoded);
        let decoded = U256::decode::<LittleEndian, 4, false>(&mut encoded.clone(), 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_be() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buf = BytesMut::new();

        original.encode::<BigEndian, 4, false>(&mut buf, 0).unwrap();

        let mut encoded = buf.freeze();
        println!("Encoded U256 (BE): {}", hex::encode(&encoded));
        let expected_encoded = "0000000000000000000000000000000000000000000000001234567890abcdef";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = U256::decode::<BigEndian, 4, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
}
