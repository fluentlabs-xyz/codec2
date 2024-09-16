use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::{Buf, BufMut, BytesMut};
use core::{mem, ptr};
use thiserror::Error;

/// ByteOrderExt is a trait that extends the functionality of the `ByteOrder` trait. It provides a method to determine if the byte order is big endian.
pub trait ByteOrderExt: ByteOrder {
    fn is_big_endian() -> bool;
}

impl ByteOrderExt for BigEndian {
    fn is_big_endian() -> bool {
        true
    }
}

impl ByteOrderExt for LittleEndian {
    fn is_big_endian() -> bool {
        false
    }
}

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Insufficient space for header: required {required} bytes, but only {available} bytes available")]
    InsufficientSpaceForHeader { required: usize, available: usize },
    #[error("Not enough space in the buffer: required {required} bytes, but only {available} bytes available")]
    BufferTooSmall {
        required: usize,
        available: usize,
        msg: String,
    },
    #[error("Invalid data encountered during decoding")]
    InvalidData,
    #[error("Not enough data in the buffer")]
    NotEnoughData,
    #[error("Unexpected end of buffer")]
    UnexpectedEof,
}

// TODO: @d1r1 Investigate whether decoding the result into an uninitialized memory (e.g., using `MaybeUninit`)
// would be more efficient than initializing with `Default`.
// This could potentially reduce unnecessary memory initialization overhead in cases where
// the default value is not required before the actual decoding takes place.
// Consider benchmarking both approaches to measure performance differences.

pub trait Encoder: Sized + Default {
    /// Header used to save metadata about the encoded value.
    const HEADER_SIZE: usize;

    /// Returns known size of the encoded value data.
    const DATA_SIZE: usize;

    /// How many bytes we should allocate for the encoded value.
    /// This is the sum of the header size and the data size.
    fn size_hint<const ALIGN: usize>(&self) -> usize {
        align_offset::<ALIGN>(Self::HEADER_SIZE + Self::DATA_SIZE)
    }

    /// Encodes the value into the given buffer at the specified offset. The buffer must be large enough to hold at least `align(offset) + Self::HEADER_SIZE` bytes.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to encode into.
    /// * `offset` - The offset in the buffer to start encoding at.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encoding was successful, or an `EncoderError` if there was a problem.
    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError>;

    /// Decodes a value from the given buffer starting at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns the decoded value if successful, or an `EncoderError` if there was a problem.
    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError>;
    // // Align the offset
    // let aligned_offset = align_offset::<ALIGN>(offset);
    // let aligned_size = align_offset::<ALIGN>(Self::HEADER_SIZE + Self::DATA_SIZE);

    // // Check if the buffer is large enough to hold the header
    // if buf.remaining() < aligned_offset + aligned_size {
    //     return Err(EncoderError::UnexpectedEof);
    // }

    // // Decode the value
    // Self::decode_inner::<B, ALIGN>(buf, aligned_offset)
    // }

    /// Decodes the header to determine the size of the encoded data.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(offset, data_length)` if successful, or an `EncoderError` if there was a problem.
    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &mut impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError>;
}

#[inline]
pub const fn align_offset<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}

// #[inline(always)]
// unsafe fn write_aligned<B: ByteOrderExt, const ALIGN: usize, T: Sized>(
//     buf: &mut impl BufMut,
//     offset: usize,
//     value: T,
// ) -> Result<(), EncoderError> {
//     let aligned_offset = align_offset::<ALIGN>(offset);
//     let value_size = mem::size_of::<T>();

// Big Endian:  [ 12 ][ 34 ][ 56 ][ 78 ]  --> Старший байт (12) в начале
// Little Endian: [ 78 ][ 56 ][ 34 ][ 12 ] --> Младший байт (78) в начале
// Value: 0x12345678
// ALIGN  = 8
// ByteOrder = little endian -> сохраняем выравнивая по левому краю
// offset = 5
// value_size = 4
// aligned_offset = 8
// buffer
//  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15
// [0, 0, 0, 0, 0, 0, 0, 0, -, -, -, -, -, -, -, -]
//               ^        ^                       ^
//               |        |                       |
//      offset = 5        |                       |
//                   aligned_offset = 8           |
//                        |                       |
//                        |   <  word size = 8 >  |
//
//                        |78,56,34,12| 0, 0, 0, 0| - little endian
//                        |0, 0, 0, 0 |12,34,56,78| - big endian
// предположим, что капасити буффера всегда достаточно для записи
// 1. Продвигаем буфер на выровненное смещение (что происходит со значением при продвижении? Оно заполняется нулями?)
// 2. В зависимости от порядка байтов в байтовом порядке мы либо пишем в буфер с начала, либо с конца
// 3. Записываем значение
// LE - записываем с начала и продигаемся до word_size - value_size
// BE - продвигаемся до word_size - value_size и записываем значения в конец

// let aligned_offset = align_offset::<ALIGN>(offset);
// let value_size = mem::size_of::<u32>();
// let total_size = align_offset::<ALIGN>(aligned_offset + value_size) - offset;

// println!("op write_aligned2");
// println!("aligned_offset: {}", aligned_offset);
// println!("value_size: {}", value_size);
// println!("total_size: {}", total_size);

// if buf.remaining_mut() < total_size {
//     return Err(EncoderError::BufferTooSmall {
//         required: total_size,
//         available: buf.remaining_mut(),
//     });
// }

// // Заполняем нулями до выровненного смещения
// for _ in offset..aligned_offset {
//     buf.put_u8(0);
// }

// // Записываем значение с учетом ByteOrder
// let mut temp_buf = [0u8; 4];
// B::write_u32(&mut temp_buf, value);
// buf.put_slice(&temp_buf);

// // Заполняем нулями оставшееся пространство
// for _ in (aligned_offset + value_size)..(offset + total_size) {
//     buf.put_u8(0);
// }

// println!("value: {}", value);
// println!("temp_buf: {:?}", temp_buf);
// println!("total_size: {}", total_size);

// Ok(())
// }

#[cfg(test)]
mod tests {
    use crate::utils::print_buffer_debug;

    use super::*;
    use byteorder::{BigEndian, LittleEndian, NativeEndian};
    use bytes::BytesMut;

    fn check_result(result: &[u8], expected: &[u8]) {
        assert_eq!(
            result, expected,
            "Result: {:?}, Expected: {:?}",
            result, expected
        );
    }

    // #[test]
    // fn test_simple_write() {
    //     let mut buf = BytesMut::with_capacity(16);
    //     let value = [1, 2, 3, 4];
    //     unsafe {
    //         write_aligned::<LittleEndian, 4>(&mut buf, 0, &value).unwrap();
    //     }
    //     check_result(&buf[..], &[1, 2, 3, 4]);
    // }

    // #[test]
    // fn test_aligned_write() {
    //     let mut buf = BytesMut::with_capacity(16);
    //     let value = [1, 2, 3, 4];
    //     unsafe {
    //         write_aligned::<LittleEndian, 8>(&mut buf, 0, &value).unwrap();
    //     }
    //     check_result(&buf[..], &[1, 2, 3, 4, 0, 0, 0, 0]);
    // }

    // #[test]
    // fn test_unaligned_write() {
    //     let mut buf = BytesMut::with_capacity(16);
    //     let value = [1, 2, 3, 4];
    //     unsafe {
    //         write_aligned::<LittleEndian, 8>(&mut buf, 1, &value).unwrap();
    //     }
    //     check_result(&buf[..], &[0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 0]);
    // }

    // #[test]
    // fn test_big_endian() {
    //     let mut buf = BytesMut::with_capacity(16);
    //     let value = 0x12345678u32.to_be_bytes();
    //     println!("expected: {:?}", &value);
    //     unsafe {
    //         write_aligned::<LittleEndian, 8>(&mut buf, 0, &value).unwrap();
    //     }
    //     println!("result: {:?}", &buf[..4]);
    //     check_result(&buf[..4], &value);
    // }
    // #[test]
    // fn test_little_endian_u32() {
    //     let mut buf = BytesMut::with_capacity(8);
    //     let value = 0x12345678u32.to_le_bytes();
    //     unsafe {
    //         write_aligned::<LittleEndian, 4>(&mut buf, 0, &value).unwrap();
    //     }
    //     assert_eq!(&buf[..4], &[0x78, 0x56, 0x34, 0x12]);
    // }

    // #[test]
    // fn test_big_endian_u32() {
    //     let value = 0x1u32;

    //     println!("le: {:?}", &value.to_le_bytes());
    //     println!("be: {:?}", &value.to_be_bytes());
    //     // le: [1, 0, 0, 0]
    //     // be: [0, 0, 0, 1]
    //     let mut buf = BytesMut::with_capacity(8);
    //     unsafe {
    //         write_aligned::<BigEndian, 4>(&mut buf, 0, value).unwrap();
    //     }
    //     println!("big endian {:?}", &buf[..]);
    //     assert_eq!(&buf[..4], value.to_be_bytes());

    //     unsafe {
    //         write_aligned::<LittleEndian, 4>(&mut buf, 4, value).unwrap();
    //     }
    //     print_buffer_debug(&buf, 0);
    //     println!("little endian {:?}", &buf[4..8]);

    //     assert_eq!(&buf[4..8], value.to_le_bytes());
    // }

    // #[test]
    // fn test_native_endian_u32() {
    //     let mut buf = BytesMut::with_capacity(8);
    //     let value = 0x12345678u32.to_ne_bytes();
    //     unsafe {
    //         write_aligned::<NativeEndian, 4>(&mut buf, 0, &value).unwrap();
    //     }
    //     println!("{:?}", &buf[..4]);
    //     println!("{:?}", &value);
    //     assert_eq!(&buf[..4], &value);
    // }

    // #[test]
    // fn test_little_endian_u64() {
    //     let mut buf = BytesMut::with_capacity(16);
    //     let value = 0x1234567890ABCDEFu64.to_le_bytes();
    //     unsafe {
    //         write_aligned::<LittleEndian, 8>(&mut buf, 0, &value).unwrap();
    //     }
    //     assert_eq!(&buf[..8], &[0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12]);
    // }

    // #[test]
    // fn test_big_endian_u64() {
    //     let mut buf = BytesMut::with_capacity(16);
    //     let value = 0x1234567890ABCDEFu64.to_be_bytes();
    //     unsafe {
    //         write_aligned::<BigEndian, 8>(&mut buf, 0, &value).unwrap();
    //     }
    //     assert_eq!(&buf[..8], &[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]);
    // }

    // // #[test]
    // // fn test_large_alignment() {
    // //     let mut buf = BytesMut::with_capacity(32);
    // //     let value = [1, 2, 3, 4];
    // //     unsafe {
    // //         write_aligned::<LittleEndian, 16>(&mut buf, 0, &value).unwrap();
    // //     }
    // //     check_result(&buf[..], &[1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    // // }

    // // #[test]
    // // fn test_small_buffer() {
    // //     let mut buf = BytesMut::with_capacity(4);
    // //     let value = [1, 2, 3, 4, 5];
    // //     let result = unsafe { write_aligned::<LittleEndian, 8>(&mut buf, 0, &value) };
    // //     assert!(result.is_err());
    // //     if let Err(EncoderError::BufferTooSmall {
    // //         required,
    // //         available,
    // //     }) = result
    // //     {
    // //         assert_eq!(required, 8);
    // //         assert_eq!(available, 4);
    // //     } else {
    // //         panic!("Ожидалась ошибка BufferTooSmall");
    // //     }
    // // }

    // // #[test]
    // // fn test_empty_value() {
    // //     let mut buf = BytesMut::with_capacity(8);
    // //     let value = [];
    // //     unsafe {
    // //         write_aligned::<LittleEndian, 4>(&mut buf, 0, &value).unwrap();
    // //     }
    // //     check_result(&buf[..], &[0, 0, 0, 0]);
    // // }

    // // #[test]
    // // fn test_large_value() {
    // //     let mut buf = BytesMut::with_capacity(32);
    // //     let value = [1; 17]; // 17 байт, больше чем 16-байтное выравнивание
    // //     unsafe {
    // //         write_aligned::<LittleEndian, 16>(&mut buf, 0, &value).unwrap();
    // //     }
    // //     let expected: Vec<u8> = [1; 17]
    // //         .into_iter()
    // //         .chain(std::iter::repeat(0).take(15))
    // //         .collect();
    // //     check_result(&buf[..], &expected);
    // // }
}
