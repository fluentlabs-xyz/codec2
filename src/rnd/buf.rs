// use bytes::{Buf, BufMut};
// use std::cmp::max;

// use crate::{
//     align::{write_slice_aligned, WritePosition},
//     encoder::{align_offset, ByteOrderExt, EncoderError},
// };

// pub fn write_bytes<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     offset: usize,
//     data_offset: usize,
//     data: &[u8],
// ) -> Result<(), EncoderError> {
//     let aligned_offset = align_offset::<ALIGN>(offset);
//     let aligned_data_offset = align_offset::<ALIGN>(data_offset);
//     let eligned_header_el = align_offset::<ALIGN>(core::mem::size_of::<u32>());

//     println!("aligned_data_offset: {}", aligned_data_offset);

//     // Calculate the total required size
//     let total_required_size = max(
//         aligned_offset + aligned_data_offset,
//         aligned_offset + 8, // Size of the header
//     ) + data.len();

//     let chunk = buf.chunk_mut();
//     println!("chunk: {:?}", chunk);
//     println!("chunk.len: {}", chunk.len());

//     // Check if the buffer has enough space for everything
//     if buf.remaining_mut() < total_required_size {
//         return Err(EncoderError::BufferTooSmall {
//             required: total_required_size,
//             available: buf.remaining_mut(),
//             msg: "Buffer too small to store all data".to_string(),
//         });
//     }

//     // Write header
//     write_header::<B, ALIGN>(
//         buf,
//         aligned_offset,
//         aligned_data_offset as u32,
//         data.len() as u32,
//     )?;

//     // Move the write position to the start of the data section
//     let advance_distance = aligned_data_offset - eligned_header_el * 2; // Subtract header size

//     println!("advance_distance: {}", advance_distance);
//     if advance_distance > 0 {
//         // Create a temporary slice to advance the buffer without modifying existing data
//         let tmp = &mut [0u8; 1024][..]; // Using a fixed-size array to avoid allocation
//         let mut remaining = advance_distance;
//         while remaining > 0 {
//             let chunk_size = remaining.min(tmp.len());
//             buf.put_slice(&tmp[..chunk_size]);
//             remaining -= chunk_size;
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
//     let write_position = if B::is_big_endian() {
//         WritePosition::End
//     } else {
//         WritePosition::Start
//     };

//     write_u32_aligned::<B, ALIGN>(buf, offset, data_offset, &write_position)?;
//     write_u32_aligned::<B, ALIGN>(buf, offset, data_length, &write_position)?;

//     Ok(())
// }

// fn write_u32_aligned<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     offset: usize,
//     value: u32,
//     write_position: &WritePosition,
// ) -> Result<(), EncoderError> {
//     let elem_size = core::mem::size_of::<u32>();
//     let mut temp = vec![0u8; 4];
//     B::write_u32(&mut temp, value);

//     unsafe { write_slice_aligned::<ALIGN>(buf, offset, &temp, write_position) }
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

// fn read_u32_aligned<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl Buf,
// ) -> Result<u32, EncoderError> {
//     let elem_size = align_offset::<ALIGN>(core::mem::size_of::<u32>());

//     if buf.remaining() < elem_size {
//         return Err(EncoderError::BufferTooSmall {
//             required: elem_size,
//             available: buf.remaining(),
//             msg: "Buffer too small to read u32".to_string(),
//         });
//     }

//     let mut temp = vec![0u8; elem_size];
//     buf.copy_to_slice(&mut temp);
//     println!("temp: {:?}", temp);

//     Ok(B::read_u32(&temp))
// }

// #[cfg(test)]
// mod tests {
//     use crate::encoder::Encoder;

//     use super::*;
//     use alloy_primitives::Bytes;
//     use byteorder::BigEndian;
//     use bytes::{buf::UninitSlice, BytesMut};
//     use core::mem::MaybeUninit;

//     #[test]
//     fn test_write_bytes123() {
//         let original = Bytes::from_static(b"Hello, World");
//         let mut buf = BytesMut::new();
//         let size_hint = original.size_hint::<8>();
//         println!("size_hint: {}", size_hint);
//         println!("original: {:?}", original.to_vec());

//         let result = write_bytes::<BigEndian, 8>(&mut buf, 0, 16, &original);

//         assert!(result.is_ok());
//         let expected = Bytes::from_static(&[
//             0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 12, 72, 101, 108, 108, 111, 44, 32, 87,
//             111, 114, 108, 100,
//         ]);

//         // print_buffer_debug(&buf, Bytes::HEADER_SIZE);

//         let mut encoded = buf.freeze();
//         assert_eq!(encoded.to_vec(), expected.to_vec());
//         // println!("Encoded: {}", hex::encode(&encoded));
//         println!("encoded: {:?}", encoded.to_vec());

//         let decoded = read_bytes::<BigEndian, 8>(&mut encoded, 0).unwrap();
//         println!("decoded: {:?}", decoded.to_vec());
//         assert_eq!(original, Bytes::from(decoded));
//     }

//     #[test]
//     fn test_write_bytes_multiple() {
//         let mut buf = BytesMut::with_capacity(64);

//         let data1 = Bytes::from_static(b"Hello");
//         let data2 = Bytes::from_static(b"World");

//         let header_el = align_offset::<8>(4);
//         let header_size = header_el * 2;
//         let data1_offset = header_size * 2; // Два заголовка
//         let data2_offset = data1_offset + align_offset::<8>(data1.len());

//         println!("data1_offset: {}", data1_offset);
//         println!("data2_offset: {}", data2_offset);
//         // Записываем первый блок данных
//         let result1 = write_bytes::<BigEndian, 8>(&mut buf, 0, data1_offset, &data1);
//         assert!(result1.is_ok());

//         // println!("buf: {:?}", buf.to_vec());

//         // Записываем второй блок данных
//         let result2 = write_bytes::<BigEndian, 8>(&mut buf, 8, data2_offset, &data2);

//         println!("result2: {:?}", result2);
//         assert!(result2.is_ok());

//         // Проверяем содержимое буфера
//         let encoded = buf.freeze();
//         println!("Encoded buffer: {:?}", encoded.to_vec());

//         // Читаем и проверяем первый блок данных
//         let mut encoded_clone = encoded.clone();
//         let decoded1 = read_bytes::<BigEndian, 8>(&mut encoded_clone, 0).unwrap();
//         println!("Decoded1: {:?}", decoded1);
//         assert_eq!(decoded1, data1.to_vec());

//         // // Читаем и проверяем второй блок данных
//         // let mut encoded_clone = encoded.clone();
//         // let decoded2 = read_bytes::<BigEndian, 8>(&mut encoded_clone, aligned_header_size).unwrap();
//         // println!("Decoded2: {:?}", decoded2);
//         // assert_eq!(decoded2, data2.to_vec());

//         // // Проверяем, что данные расположены после всех заголовков
//         // assert_eq!(
//         //     &encoded[data1_offset..data1_offset + data1.len()],
//         //     data1.as_ref()
//         // );
//         // assert_eq!(
//         //     &encoded[data2_offset..data2_offset + data2.len()],
//         //     data2.as_ref()
//         // );
//     }
// }
