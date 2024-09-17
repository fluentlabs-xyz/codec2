// use bytes::{Buf, BufMut, Bytes, BytesMut};
// use std::cmp::max;

// use crate::encoder::{align_offset, ByteOrderExt, EncoderError};

// pub fn write_bytes_unsafe<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     header_offset: usize,
//     data_offset: usize,
//     data: &[u8],
// ) -> Result<(), EncoderError> {
//     let aligned_header_offset = align_offset::<ALIGN>(header_offset);
//     let aligned_data_offset = align_offset::<ALIGN>(data_offset);
//     let aligned_header_size = align_offset::<ALIGN>(core::mem::size_of::<u32>()) * 2;

//     let total_required_size = max(
//         aligned_data_offset + data.len(),
//         aligned_header_offset + aligned_header_size,
//     );

//     if buf.remaining_mut() < total_required_size {
//         return Err(EncoderError::BufferTooSmall {
//             required: total_required_size,
//             available: buf.remaining_mut(),
//             msg: "Buffer too small to store all data".to_string(),
//         });
//     }

//     unsafe {
//         let buf_ptr = buf.chunk_mut().as_mut_ptr();
//         let buf_len = buf.chunk_mut().len();

//         println!("buf_ptr: {:p}", buf_ptr);
//         println!("buf_len: {}", buf_len);

//         // Write header
//         let header_ptr = buf_ptr.add(aligned_header_offset);
//         // write_header_raw::<B, ALIGN>(header_ptr, aligned_data_offset as u32, data.len() as u32)?;

//         // Write data
//         let data_ptr = buf_ptr.add(aligned_data_offset);
//         std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());

//         // Update the buffer's internal write position
//         let new_len = max(buf_len, total_required_size);
//         buf.advance_mut(new_len - buf_len);
//         // // Сохраняем текущую позицию
//         // let original_len =
//         //     buf.chunk_mut().as_mut_ptr() as usize - buf.chunk_mut().as_mut_ptr() as usize;

//         // println!("original_len: {}", original_len);
//         // // Перемещаемся к позиции заголовка
//         // buf.advance_mut(aligned_header_offset.saturating_sub(original_len));

//         // // Записываем заголовок
//         // write_header::<B, ALIGN>(buf, 0, aligned_data_offset as u32, data.len() as u32)?;
//         // let data_offset =
//         //     aligned_data_offset.saturating_sub(aligned_header_offset + aligned_header_size);
//         // println!("data_offset: {}", data_offset);
//         // println!("aligned_data_offset: {}", aligned_data_offset);
//         // println!("aligned_header_offset: {}", aligned_header_offset);
//         // println!("aligned_header_size: {}", aligned_header_size);

//         // // Перемещаемся к позиции данных
//         // buf.advance_mut(
//         //     aligned_data_offset.saturating_sub(aligned_header_offset + aligned_header_size),
//         // );

//         // // Записываем данные
//         // buf.put_slice(data);

//         // // Возвращаемся в конец буфера
//         // let new_len = buf.chunk_mut().as_mut_ptr() as usize - buf.chunk_mut().as_mut_ptr() as usize;
//         // if new_len < original_len {
//         //     buf.advance_mut(original_len - new_len);
//         // }
//     }

//     Ok(())
// }

// pub fn write_bytes<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     offset: usize,
//     data_offset: usize,
//     data: &[u8],
// ) -> Result<(), EncoderError> {
//     let aligned_offset = align_offset::<ALIGN>(offset);
//     let aligned_data_offset = align_offset::<ALIGN>(data_offset);
//     let aligned_header_size = align_offset::<ALIGN>(core::mem::size_of::<u32>()) * 2;

//     // Calculate the total required size
//     let total_required_size = max(
//         aligned_offset + aligned_data_offset,
//         aligned_offset + aligned_header_size,
//     ) + data.len();

//     // Check if the buffer has enough space for everything
//     if buf.remaining_mut() < total_required_size {
//         return Err(EncoderError::BufferTooSmall {
//             required: total_required_size,
//             available: buf.remaining_mut(),
//             msg: "Buffer too small to store all data".to_string(),
//         });
//     }

//     println!("aligned_offset: {}", aligned_offset);
//     println!("aligned_data_offset: {}", aligned_data_offset);

//     // Write header
//     write_header::<B, ALIGN>(
//         buf,
//         aligned_offset,
//         aligned_data_offset as u32,
//         data.len() as u32,
//     )?;

//     // Move the write position to the start of the data section
//     let advance_distance = aligned_data_offset - aligned_header_size;
//     if advance_distance > 0 {
//         let current_len = buf.chunk_mut().len();
//         if current_len >= aligned_offset + aligned_data_offset {
//             // Data exists, just advance the position
//             unsafe {
//                 buf.advance_mut(advance_distance);
//             }
//         } else {
//             // No data, fill with zeros
//             buf.put_bytes(0, advance_distance);
//         }
//     }

//     // Write data
//     buf.put_slice(data);

//     Ok(())
// }

// fn write_header<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     offset: usize,
//     data_offset: u32,
//     data_length: u32,
// ) -> Result<(), EncoderError> {
//     write_u32_aligned::<B, ALIGN>(buf, offset, data_offset)?;
//     write_u32_aligned::<B, ALIGN>(buf, offset, data_length)?;
//     Ok(())
// }

// fn write_u32_aligned<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     offset: usize,
//     value: u32,
// ) -> Result<(), EncoderError> {
//     let mut temp = [0u8; 4];
//     B::write_u32(&mut temp, value);
//     let aligned = align::<B, ALIGN>(&temp);

//     // Ensure the buffer has enough space
//     if buf.remaining_mut() < offset + aligned.len() {
//         return Err(EncoderError::BufferTooSmall {
//             required: offset + aligned.len(),
//             available: buf.remaining_mut(),
//             msg: "Buffer too small to write aligned u32".to_string(),
//         });
//     }
//     // Create a temporary buffer to handle the offset
//     let mut temp_buf = vec![0u8; offset + aligned.len()];
//     temp_buf[offset..].copy_from_slice(&aligned);
//     println!("temp_buf: {:?}", temp_buf);

//     // Write the entire temporary buffer
//     buf.put_slice(&temp_buf);

//     Ok(())
// }

// pub fn read_bytes<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl Buf,
//     offset: usize,
// ) -> Result<Vec<u8>, EncoderError> {
//     let (data_offset, data_length) = read_bytes_header::<B, ALIGN>(buf, offset)?;

//     println!("data_offset: {}", data_offset);
//     println!("data_length: {}", data_length);

//     if buf.remaining() < data_offset + data_length as usize {
//         return Err(EncoderError::BufferTooSmall {
//             required: data_offset + data_length as usize,
//             available: buf.remaining(),
//             msg: "Buffer too small to read data".to_string(),
//         });
//     }

//     println!(">>>data: {:?}", buf.chunk()[..].to_vec());
//     println!(">>>data_offset: {}", data_offset);
//     let data = buf.chunk()[data_offset..].to_vec();
//     println!(">>data: {:?}", data);

//     Ok(data)
// }

// pub fn read_bytes_header<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl Buf,
//     offset: usize,
// ) -> Result<(usize, usize), EncoderError> {
//     println!("op read_bytes_header");
//     let aligned_offset = align_offset::<ALIGN>(offset);
//     let elem_size = align_offset::<ALIGN>(core::mem::size_of::<u32>());

//     let header_size = elem_size * 2;

//     if buf.remaining() < aligned_offset + header_size {
//         return Err(EncoderError::BufferTooSmall {
//             required: aligned_offset + header_size,
//             available: buf.remaining(),
//             msg: "Buffer too small to read header".to_string(),
//         });
//     }

//     if B::is_big_endian() {
//         let offset = B::read_u32(&buf.chunk()[elem_size - 4..elem_size]) as usize;
//         let length = B::read_u32(&buf.chunk()[elem_size * 2 - 4..elem_size * 2]) as usize;
//         Ok((offset, length))
//     } else {
//         let offset = B::read_u32(&buf.chunk()[..elem_size]) as usize;
//         let length = B::read_u32(&buf.chunk()[elem_size..elem_size * 2]) as usize;
//         Ok((offset, length))
//     }
// }

// /// Aligns the source bytes to the specified alignment.
// pub fn align<B: ByteOrderExt, const ALIGN: usize>(src: &[u8]) -> Bytes {
//     let aligned_src_len = align_offset::<ALIGN>(src.len());
//     let aligned_total_size = aligned_src_len.max(ALIGN);
//     let mut aligned = BytesMut::zeroed(aligned_total_size);

//     if B::is_big_endian() {
//         // For big-endian, copy to the end of the aligned array
//         let start = aligned_total_size - src.len();
//         aligned[start..].copy_from_slice(src);
//     } else {
//         // For little-endian, copy to the start of the aligned array
//         aligned[..src.len()].copy_from_slice(src);
//     }

//     aligned.freeze()
// }

// #[cfg(test)]
// mod tests {
//     use crate::encoder::Encoder;

//     use super::*;
//     use alloy_primitives::Bytes;
//     use byteorder::{BigEndian, LittleEndian};
//     use bytes::{buf::UninitSlice, BytesMut};
//     use core::mem::MaybeUninit;

//     #[test]
//     fn test_align_overflow() {
//         // Test for big-endian
//         let src = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
//         let aligned = align::<BigEndian, 8>(&src);
//         assert_eq!(
//             aligned.to_vec(),
//             [0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//         );

//         // Test for little-endian
//         let aligned = align::<LittleEndian, 8>(&src);
//         println!("aligned: {:?}", aligned.to_vec());
//         assert_eq!(
//             aligned.to_vec(),
//             [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0, 0]
//         );
//     }

//     #[test]
//     fn test_write_bytes1234() {
//         let original = Bytes::from_static(b"Hello, World");
//         let mut buf = BytesMut::new();
//         let size_hint = original.size_hint::<8>();

//         let result = write_bytes_unsafe::<BigEndian, 8>(&mut buf, 0, 16, &original);

//         assert!(result.is_ok());
//         let expected = Bytes::from_static(&[
//             0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 12, 72, 101, 108, 108, 111, 44, 32, 87,
//             111, 114, 108, 100,
//         ]);

//         let mut encoded = buf.freeze();
//         assert_eq!(encoded.to_vec(), expected.to_vec());

//         let decoded = read_bytes::<BigEndian, 8>(&mut encoded, 0).unwrap();

//         assert_eq!(original, Bytes::from(decoded));
//     }
//     #[test]
//     fn test_write_to_exising_buf() {
//         let original = Bytes::from_static(b"Hello, World");
//         let mut buf = BytesMut::new();
//         buf.extend_from_slice(&[
//             0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//             0, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100,
//         ]);
//         println!("buf: {:?}", &buf.to_vec());

//         let result = write_bytes_unsafe::<BigEndian, 8>(&mut buf, 0, 32, &original);

//         assert!(result.is_ok());

//         println!("buf: {:?}", &buf.to_vec());

//         let expected = [
//             0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0,
//             0, 0, 0, 12, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 72, 101, 108, 108,
//             111, 44, 32, 87, 111, 114, 108, 100,
//         ];

//         assert_eq!(buf.to_vec(), expected);
//     }
// }
