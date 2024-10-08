use crate::{
    bytes::{read_bytes, read_bytes_header, read_bytes_header_solidity, write_bytes},
    encoder::{
        align_up,
        get_aligned_slice,
        is_big_endian,
        read_u32_aligned,
        write_u32_aligned,
        Encoder,
    },
    error::{CodecError, DecodingError},
};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};
use std::usize;

impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for Bytes {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;
    const IS_DYNAMIC: bool = true;
    /// Encode the bytes into the buffer.
    /// First, we encode the header and write it to the given offset.
    /// After that, we encode the actual data and write it to the end of the buffer.
    /// Note, for Solidity we need to write offset = actual_data_offset - 32.
    /// But if offset is 0, we need to write 32.
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let elem_size = align_up::<ALIGN>(4);

        // Ensure the buffer has enough space for the offset + header size
        if buf.len() < aligned_offset + elem_size {
            buf.resize(aligned_offset + elem_size, 0);
        }

        // Write the offset of the data (current length of the buffer)
        let current_len = buf.len() as u32;
        let encoded_offset = if offset == 0 {
            32 // Special case when offset is 0, we write 32 as required by ABI
        } else {
            current_len - 32 // Write actual_data_offset - 32
        };
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, encoded_offset);

        // Write the actual data to the buffer at the current length
        let data_start = buf.len(); // Where the actual data will start
        let _ = write_bytes::<B, ALIGN, true>(buf, data_start, self, self.len() as u32);

        // Add padding if necessary to ensure the buffer remains aligned
        if buf.len() % ALIGN != 0 {
            let padding = ALIGN - (buf.len() % ALIGN);
            buf.resize(buf.len() + padding, 0);
        }

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        println!("op.bytes.decode.sol_mode");
        println!("offset: {:?}", offset);

        let (data_offset, data_len) = read_bytes_header::<B, ALIGN, true>(buf, offset)?;
        println!(">>>Data offset: {}, Data length: {}", data_offset, data_len);

        let data = read_bytes::<B, ALIGN, true>(buf, offset)?;

        println!("data: {:?}", &data.chunk()[..]);

        Ok(Self::from(data))
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        read_bytes_header_solidity::<B, ALIGN>(buf, offset)
    }
}

impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for Bytes {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;
    const IS_DYNAMIC: bool = true;
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let elem_size = align_up::<ALIGN>(4);
        if buf.len() < aligned_offset + elem_size {
            buf.resize(aligned_offset + elem_size, 0);
        }

        // Write the offset of the data (current length of the buffer)
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, buf.len() as u32);

        // Write actual data
        let _ = write_bytes::<B, ALIGN, false>(buf, aligned_offset, self, self.len() as u32);

        // Ensure the buffer is aligned
        if buf.len() % ALIGN != 0 {
            let padding = ALIGN - (buf.len() % ALIGN);
            buf.resize(buf.len() + padding, 0);
        }

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        let (data_offset, _data_size) =
            <Self as Encoder<B, { ALIGN }, false>>::partial_decode(buf, aligned_offset)?;

        let data = read_bytes::<B, ALIGN, false>(buf, data_offset)?;

        Ok(Self::from(data))
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);

        let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
        let data_size = read_u32_aligned::<B, ALIGN>(buf, aligned_offset + 4)? as usize;
        Ok((data_offset, data_size))
    }
}

// Implementation for SOL_MODE = false
impl<const N: usize, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false>
    for FixedBytes<N>
{
    const HEADER_SIZE: usize = N;
    const IS_DYNAMIC: bool = false;
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let slice = get_aligned_slice::<B, ALIGN>(buf, aligned_offset, N);
        slice.copy_from_slice(self.as_ref());
        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + N {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + N,
                found: buf.remaining(),
                msg: "Buffer too small to decode FixedBytes".to_string(),
            }));
        }
        let data = buf.chunk()[aligned_offset..aligned_offset + N].to_vec();
        Ok(FixedBytes::from_slice(&data))
    }

    fn partial_decode(_buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        Ok((aligned_offset, N))
    }
}

// Implementation for SOL_MODE = true
impl<const N: usize, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true>
    for FixedBytes<N>
{
    const HEADER_SIZE: usize = 32; // Always 32 bytes for Solidity ABI
    const IS_DYNAMIC: bool = false;
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<32>(offset); // Always 32-byte aligned for Solidity
        let slice = get_aligned_slice::<B, 32>(buf, aligned_offset, 32);
        slice[..N].copy_from_slice(self.as_ref());
        // Zero-pad the rest
        slice[N..].fill(0);
        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<32>(offset); // Always 32-byte aligned for Solidity
        if buf.remaining() < aligned_offset + 32 {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + 32,
                found: buf.remaining(),
                msg: "Buffer too small to decode FixedBytes".to_string(),
            }));
        }
        let data = buf.chunk()[aligned_offset..aligned_offset + N].to_vec();
        Ok(FixedBytes::from_slice(&data))
    }

    fn partial_decode(_buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<32>(offset); // Always 32-byte aligned for Solidity
        Ok((aligned_offset, 32))
    }
}
macro_rules! impl_evm_fixed {
    ($typ:ty) => {
        impl<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>
            Encoder<B, { ALIGN }, { SOL_MODE }> for $typ
        {
            const HEADER_SIZE: usize = if SOL_MODE { 32 } else { <$typ>::len_bytes() };
            const IS_DYNAMIC: bool = false;
            fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
                let aligned_offset = if SOL_MODE {
                    align_up::<32>(offset)
                } else {
                    align_up::<ALIGN>(offset)
                };
                let word_size = if SOL_MODE {
                    32
                } else {
                    align_up::<ALIGN>(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
                };

                let slice = get_aligned_slice::<B, { ALIGN }>(buf, aligned_offset, word_size);
                let bytes: &[u8] = self.0.as_ref();

                if SOL_MODE {
                    // For Solidity ABI, right-align the data
                    slice[word_size - Self::len_bytes()..].copy_from_slice(bytes);
                    slice[..word_size - Self::len_bytes()].fill(0); // Zero-pad the rest
                } else if is_big_endian::<B>() {
                    slice[word_size - Self::len_bytes()..].copy_from_slice(bytes);
                } else {
                    slice[..Self::len_bytes()].copy_from_slice(bytes);
                }

                Ok(())
            }

            fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
                let aligned_offset = if SOL_MODE {
                    align_up::<32>(offset)
                } else {
                    align_up::<ALIGN>(offset)
                };
                let word_size = if SOL_MODE {
                    32
                } else {
                    align_up::<ALIGN>(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
                };

                if buf.remaining() < aligned_offset + word_size {
                    return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + word_size,
                        found: buf.remaining(),
                        msg: format!("buf too small to read aligned {}", stringify!($typ)),
                    }));
                }

                let chunk = &buf.chunk()[aligned_offset..aligned_offset + word_size];

                let inner = if SOL_MODE || is_big_endian::<B>() {
                    FixedBytes::<{ Self::len_bytes() }>::from_slice(
                        &chunk[word_size - Self::len_bytes()..],
                    )
                } else {
                    FixedBytes::<{ Self::len_bytes() }>::from_slice(&chunk[..Self::len_bytes()])
                };

                Ok(Self(inner))
            }

            fn partial_decode(
                _buf: &impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                let aligned_offset = if SOL_MODE {
                    align_up::<32>(offset)
                } else {
                    align_up::<ALIGN>(offset)
                };
                let word_size = if SOL_MODE {
                    32
                } else {
                    align_up::<ALIGN>(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
                };
                Ok((aligned_offset, word_size))
            }
        }
    };
}

impl_evm_fixed!(Address);
impl<
        const BITS: usize,
        const LIMBS: usize,
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for Uint<BITS, LIMBS>
{
    const HEADER_SIZE: usize = if SOL_MODE { 32 } else { Self::BYTES };
    const IS_DYNAMIC: bool = false;

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = if SOL_MODE {
            align_up::<32>(offset)
        } else {
            align_up::<ALIGN>(offset)
        };
        let word_size = if SOL_MODE {
            32
        } else {
            align_up::<ALIGN>(Self::BYTES)
        };

        let slice = get_aligned_slice::<B, { ALIGN }>(buf, aligned_offset, word_size);

        let bytes = if is_big_endian::<B>() {
            self.to_be_bytes_vec()
        } else {
            self.to_le_bytes_vec()
        };

        if SOL_MODE {
            // For Solidity ABI, right-align the data
            slice[word_size - Self::BYTES..].copy_from_slice(&bytes);
            slice[..word_size - Self::BYTES].fill(0); // Zero-pad the rest
        } else {
            slice[..Self::BYTES].copy_from_slice(&bytes);
        }

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let aligned_offset = if SOL_MODE {
            align_up::<32>(offset)
        } else {
            align_up::<ALIGN>(offset)
        };
        let word_size = if SOL_MODE {
            32
        } else {
            align_up::<ALIGN>(Self::BYTES)
        };

        if buf.remaining() < aligned_offset + word_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + word_size,
                found: buf.remaining(),
                msg: "buf too small to read Uint".to_string(),
            }));
        }

        let chunk = &buf.chunk()[aligned_offset..aligned_offset + word_size];
        let value_slice = if SOL_MODE {
            &chunk[word_size - Self::BYTES..]
        } else {
            &chunk[..Self::BYTES]
        };

        let value = if is_big_endian::<B>() {
            Self::from_be_slice(value_slice)
        } else {
            Self::from_le_slice(value_slice)
        };

        Ok(value)
    }

    fn partial_decode(_buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        let aligned_offset = if SOL_MODE {
            align_up::<32>(offset)
        } else {
            align_up::<ALIGN>(offset)
        };
        let word_size = if SOL_MODE {
            32
        } else {
            align_up::<ALIGN>(Self::BYTES)
        };
        Ok((aligned_offset, word_size))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[cfg(test)]
    use alloy_primitives::{Address, U256};
    use byteorder::{BigEndian, LittleEndian};
    use bytes::BytesMut;

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
        let _result =
            write_bytes::<BigEndian, 8, false>(&mut buf, 16, &original, original.len() as u32);

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

        println!("Decoded Bytes: {:?}", decoded.to_vec());
        assert_eq!(decoded.to_vec()[12..], original.to_vec());
    }

    #[test]
    fn test_address_encode_decode() {
        let original = Address::from([0x42; 20]);
        let mut buf = BytesMut::new();

        <Address as Encoder<LittleEndian, 1, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("Encoded Address: {}", hex::encode(&encoded));

        let decoded = <Address as Encoder<LittleEndian, 1, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_address_encode_decode_aligned() {
        let original = Address::from([0x42; 20]);
        let mut buf = BytesMut::new();

        <Address as Encoder<LittleEndian, 32, true>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("Encoded Address: {}", hex::encode(&encoded));

        let decoded = <Address as Encoder<LittleEndian, 32, true>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_le() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buf = BytesMut::new();

        <U256 as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("Encoded U256 (LE): {}", hex::encode(&encoded));
        let expected_encoded = "efcdab9078563412000000000000000000000000000000000000000000000000";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = <U256 as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_uint_encode_decode_be() {
        let original = U256::from(0x1234567890abcdef_u64);
        let mut buf = BytesMut::new();

        <U256 as Encoder<BigEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("Encoded U256 (BE): {}", hex::encode(&encoded));
        let expected_encoded = "0000000000000000000000000000000000000000000000001234567890abcdef";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = <U256 as Encoder<BigEndian, 4, false>>::decode(&encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
}
