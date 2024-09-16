extern crate alloc;

use alloc::slice;

use crate::{
    align::{self, write_slice_aligned, WritePosition},
    encoder::{align_offset, ByteOrderExt, Encoder, EncoderError},
};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::mem::MaybeUninit;

impl Encoder for u8 {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = core::mem::size_of::<u8>();

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError> {
        // Align the offset and header size
        let aligned_offset = align_offset::<ALIGN>(offset);

        // How many bytes we need to store the header and known data size
        let aligned_size_hint = &self.size_hint::<ALIGN>();
        println!("aligned_size_hint: {}", aligned_size_hint);

        // Check if the buffer is large enough to hold the header
        if buf.remaining_mut() < aligned_offset + aligned_size_hint {
            return Err(EncoderError::InsufficientSpaceForHeader {
                required: aligned_offset + aligned_size_hint,
                available: buf.remaining_mut(),
            });
        }

        // Encode the value

        let write_postion = match B::is_big_endian() {
            true => WritePosition::End,
            false => WritePosition::Start,
        };
        unsafe {
            write_slice_aligned::<ALIGN>(buf, offset, &[*self], write_postion)?;
        }

        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + ALIGN {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(aligned_offset);
        let chunk = buf.chunk();
        let value = if B::is_big_endian() {
            chunk[ALIGN - 1]
        } else {
            chunk[0]
        };
        buf.advance(ALIGN);

        Ok(value)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + Self::DATA_SIZE {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(offset);

        Ok((offset, Self::DATA_SIZE))
    }
}

impl Encoder for bool {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = core::mem::size_of::<bool>();

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError> {
        let write_position = if B::is_big_endian() {
            WritePosition::End
        } else {
            WritePosition::Start
        };

        let byte = if *self { 1u8 } else { 0u8 };
        unsafe {
            write_slice_aligned::<ALIGN>(buf, offset, &[byte], write_position)?;
        }

        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + ALIGN {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(aligned_offset);
        let chunk = buf.chunk();
        let byte = if B::is_big_endian() {
            chunk[ALIGN - 1]
        } else {
            chunk[0]
        };
        buf.advance(ALIGN);

        Ok(byte != 0)
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + Self::DATA_SIZE {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(offset);

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
                buf: &mut impl BufMut,
                offset: usize,
            ) -> Result<(), EncoderError> {
                let mut bytes = [0u8; Self::DATA_SIZE];

                B::$write_method(&mut bytes, *self);

                let write_position = if B::is_big_endian() {
                    WritePosition::End
                } else {
                    WritePosition::Start
                };

                unsafe {
                    write_slice_aligned::<ALIGN>(buf, offset, &bytes, write_position)?;
                }

                Ok(())
            }

            fn decode<B: ByteOrderExt, const ALIGN: usize>(
                buf: &mut impl Buf,
                offset: usize,
            ) -> Result<Self, EncoderError> {
                let aligned_offset = align_offset::<ALIGN>(offset);
                if buf.remaining() < aligned_offset + ALIGN {
                    return Err(EncoderError::NotEnoughData);
                }

                buf.advance(aligned_offset);
                let chunk = buf.chunk();
                let value = if B::is_big_endian() {
                    B::$read_method(&chunk[ALIGN - Self::DATA_SIZE..ALIGN])
                } else {
                    B::$read_method(&chunk[..Self::DATA_SIZE])
                };
                buf.advance(ALIGN);

                Ok(value)
            }

            fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
                buf: &mut impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), EncoderError> {
                let aligned_offset = align_offset::<ALIGN>(offset);
                if buf.remaining() < aligned_offset + Self::DATA_SIZE {
                    return Err(EncoderError::NotEnoughData);
                }

                Ok((aligned_offset, Self::DATA_SIZE))
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

impl<T: Sized + Encoder + Default> Encoder for Option<T> {
    const HEADER_SIZE: usize = 1 + T::HEADER_SIZE;
    const DATA_SIZE: usize = T::DATA_SIZE;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        let aligned_header_size = align_offset::<ALIGN>(Self::HEADER_SIZE);
        let aligned_data_size = align_offset::<ALIGN>(T::DATA_SIZE);

        let required_space = aligned_offset + aligned_header_size + aligned_data_size;

        if buf.remaining_mut() < required_space {
            return Err(EncoderError::InsufficientSpaceForHeader {
                required: required_space,
                available: buf.remaining_mut(),
            });
        }

        let write_position = if B::is_big_endian() {
            WritePosition::End
        } else {
            WritePosition::Start
        };
        let option_flag = if self.is_some() { 1 } else { 0 };

        unsafe {
            write_slice_aligned::<ALIGN>(buf, offset, &[option_flag], write_position)?;
        };

        if let Some(inner_value) = self {
            inner_value.encode::<B, ALIGN>(buf, aligned_offset)?;
        } else {
            let default_value = T::default();
            default_value.encode::<B, ALIGN>(buf, aligned_offset)?;
        };
        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        let aligned_header_size = align_offset::<ALIGN>(Self::HEADER_SIZE);
        let aligned_data_size = align_offset::<ALIGN>(T::DATA_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size + aligned_data_size {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(aligned_offset);

        let option_flag = if B::is_big_endian() {
            buf.chunk()[ALIGN - 1]
        } else {
            buf.chunk()[0]
        };
        buf.advance(ALIGN);

        if option_flag != 0 {
            let inner_value = T::decode::<B, ALIGN>(buf, 0)?;
            Ok(Some(inner_value))
        } else {
            buf.advance(aligned_data_size);
            Ok(None)
        }
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        if buf.remaining() < aligned_offset + Self::HEADER_SIZE {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(aligned_offset);
        let option_flag = buf.get_u8();

        if option_flag != 0 {
            let (_, inner_size) = T::partial_decode::<B, ALIGN>(buf, 0)?;
            Ok((aligned_offset, Self::HEADER_SIZE + inner_size))
        } else {
            Ok((aligned_offset, Self::HEADER_SIZE + T::DATA_SIZE))
        }
    }
}

pub struct ArrayWrapper<T, const N: usize>([T; N]);

impl<T: Default, const N: usize> Default for ArrayWrapper<T, N> {
    fn default() -> Self {
        ArrayWrapper(core::array::from_fn(|_| T::default()))
    }
}

impl<T, const N: usize> ArrayWrapper<T, N> {
    pub fn new(arr: [T; N]) -> Self {
        ArrayWrapper(arr)
    }

    pub fn into_inner(self) -> [T; N] {
        self.0
    }
}

impl<T, const N: usize> Encoder for ArrayWrapper<T, N>
where
    T: Sized + Encoder + Default,
{
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = T::DATA_SIZE * N;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        let aligned_element_size = align_offset::<ALIGN>(T::DATA_SIZE);
        let total_size = N * aligned_element_size;

        println!("Encoding ArrayWrapper:");
        println!("  Aligned offset: {}", aligned_offset);
        println!("  Aligned element size: {}", aligned_element_size);
        println!("  Total size: {}", total_size);
        println!("  Buffer remaining: {}", buf.remaining_mut());

        if buf.remaining_mut() < aligned_offset + total_size {
            return Err(EncoderError::InsufficientSpaceForHeader {
                required: aligned_offset + total_size,
                available: buf.remaining_mut(),
            });
        }

        // Заполняем выравнивание нулями, если необходимо
        buf.put_bytes(0, aligned_offset);

        for (i, item) in self.0.iter().enumerate() {
            println!(
                "Encoding item {} at offset {}",
                i,
                aligned_offset + i * aligned_element_size
            );
            item.encode::<B, ALIGN>(buf, 0)?;

            // Добавляем padding после каждого элемента, если необходимо
            let padding = aligned_element_size - T::DATA_SIZE;
            if padding > 0 {
                buf.put_bytes(0, padding);
            }
        }

        println!("Encoding completed. Buffer size: {}", buf.remaining_mut());

        Ok(())
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        let aligned_element_size = align_offset::<ALIGN>(T::DATA_SIZE);
        let total_size = N * aligned_element_size;

        println!("Decoding ArrayWrapper:");
        println!("  Aligned offset: {}", aligned_offset);
        println!("  Aligned element size: {}", aligned_element_size);
        println!("  Total size: {}", total_size);
        println!("  Buffer remaining: {}", buf.remaining());

        if buf.remaining() < aligned_offset + total_size {
            return Err(EncoderError::NotEnoughData);
        }

        buf.advance(aligned_offset);

        let result = core::array::from_fn(|i| {
            println!("Decoding item {} at offset {}", i, i * aligned_element_size);
            let decoded = T::decode::<B, ALIGN>(buf, 0).unwrap_or_else(|_| T::default());

            // Пропускаем padding после каждого элемента, если есть
            let padding = aligned_element_size - T::DATA_SIZE;
            if padding > 0 {
                buf.advance(padding);
            }

            decoded
        });

        println!("Decoding completed. Buffer remaining: {}", buf.remaining());

        Ok(ArrayWrapper(result))
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError> {
        let aligned_offset = align_offset::<ALIGN>(offset);
        let aligned_element_size = align_offset::<ALIGN>(T::DATA_SIZE);
        let total_size = N * aligned_element_size;

        if buf.remaining() < aligned_offset + total_size {
            return Err(EncoderError::NotEnoughData);
        }

        Ok((aligned_offset, total_size))
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::utils::print_buffer_debug;
    use alloc::vec::Vec;
    use byteorder::{BigEndian, LittleEndian};
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_u8_encode_decode() {
        let original: u8 = 1;
        let mut buffer = BytesMut::with_capacity(32);

        println!("Buffer capacity: {}", buffer.capacity());

        let is_ok = original.encode::<BigEndian, 32>(&mut buffer, 0);
        assert!(is_ok.is_ok());

        print_buffer_debug(&buffer, 0);

        println!("{:?}", hex::encode(&buffer));

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = u8::decode::<BigEndian, 32>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
        println!("encoded: {:?}", buffer);

        let partial_decoded =
            u8::partial_decode::<BigEndian, 32>(&mut buffer.clone().freeze(), 0).unwrap();
        assert_eq!(partial_decoded, (0, 1));
    }

    #[test]
    fn test_bool_encode_decode_align8() {
        let original = true;
        let mut buffer = BytesMut::with_capacity(8);

        original.encode::<LittleEndian, 8>(&mut buffer, 0).unwrap();

        assert_eq!(buffer.to_vec(), vec![1, 0, 0, 0, 0, 0, 0, 0]);

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = bool::decode::<LittleEndian, 8>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_encode_decode_le() {
        let original: u32 = 0x12345678;
        let mut buffer = BytesMut::with_capacity(4);

        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        assert_eq!(buffer.to_vec(), vec![0x78, 0x56, 0x34, 0x12]);

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = u32::decode::<LittleEndian, 4>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u64_encode_decode_be() {
        let original: u64 = 0x1234567890ABCDEF;
        let mut buffer = BytesMut::with_capacity(8);

        original.encode::<BigEndian, 8>(&mut buffer, 0).unwrap();

        assert_eq!(
            buffer.to_vec(),
            vec![0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]
        );

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = u64::decode::<BigEndian, 8>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_i32_encode_decode_le() {
        let original: i32 = -123456;
        let mut buffer = BytesMut::with_capacity(4);

        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        let mut buf_for_decode = buffer.clone().freeze();
        let decoded = i32::decode::<LittleEndian, 4>(&mut buf_for_decode, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_u32_encode_decode() {
        let original: Option<u32> = Some(0x12345678);
        let mut buffer = BytesMut::with_capacity(8);

        let ok = original.encode::<LittleEndian, 4>(&mut buffer, 0);
        assert!(ok.is_ok());

        let mut encoded = buffer.freeze();
        assert_eq!(
            encoded,
            Bytes::from_static(&[1, 0x00, 0x00, 0x00, 0x78, 0x56, 0x34, 0x12])
        );

        let decoded = Option::<u32>::decode::<LittleEndian, 4>(&mut encoded, 0);

        assert_eq!(original, decoded.unwrap());
    }

    #[test]
    fn test_array_wrapper_u16_encode_decode() {
        let original = ArrayWrapper::new([0x1234u16, 0x5678u16, 0x9ABCu16]);
        const ALIGNMENT: usize = 2;

        println!("Original array: {:?}", original.0);
        println!("Size hint: {}", original.size_hint::<ALIGNMENT>());
        let mut buffer = BytesMut::with_capacity(original.size_hint::<ALIGNMENT>());
        println!("Buffer capacity: {}", buffer.capacity());

        original
            .encode::<BigEndian, ALIGNMENT>(&mut buffer, 0)
            .unwrap();

        println!("Encoded buffer:");
        print_buffer_debug(&buffer, 0);

        assert_eq!(buffer.len(), 6, "Buffer length should be 6");

        let expected = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
        assert_eq!(
            buffer.to_vec(),
            expected,
            "Encoded data does not match expected values. Expected: {:?}, Got: {:?}",
            expected,
            buffer.to_vec()
        );

        let mut buf_for_decode = buffer.freeze();
        println!("Buffer for decode: {:?}", buf_for_decode);

        let decoded =
            ArrayWrapper::<u16, 3>::decode::<BigEndian, ALIGNMENT>(&mut buf_for_decode, 0).unwrap();

        println!("Decoded array: {:?}", decoded.0);

        assert_eq!(
            original.into_inner(),
            decoded.into_inner(),
            "Decoded array does not match original"
        );
    }
}
