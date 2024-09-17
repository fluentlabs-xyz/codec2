// use byteorder::BigEndian;

// use bytes::{Buf, BufMut, Bytes, BytesMut};
// use std::cmp::max;
// use std::ops::{Index, IndexMut};

// use crate::encoder::{align_offset, ByteOrderExt, EncoderError};

// trait IndexableBufMut: BufMut + Index<usize, Output = u8> + IndexMut<usize> {
//     fn len(&self) -> usize;
//     fn resize(&mut self, new_len: usize, value: u8);
// }

// pub fn write_bytes_unsafe<B: ByteOrderExt, const ALIGN: usize>(
//     buf: &mut impl BufMut,
//     header_offset: usize,
//     data_offset: usize,
//     data: &[u8],
// ) -> Result<(), EncoderError> {
//     let aligned_header_offset = align_offset::<ALIGN>(header_offset);
//     let aligned_data_offset = align_offset::<ALIGN>(data_offset);
//     let aligned_header_size = align_offset::<ALIGN>(core::mem::size_of::<u32>() * 2);

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

//         // let buf_len = buf.chunk_mut().len();

//         // // println!("buf_ptr: {:p}", buf_ptr);
//         // // println!("buf_len: {}", buf_len);
//         // // println!("aligned_header_offset: {}", aligned_header_offset);
//         // // println!("aligned_data_offset: {}", aligned_data_offset);

//         // // Write header
//         // let header_ptr = buf_ptr.sub(buf_len);

//         // std::ptr::copy_nonoverlapping(data.as_ptr(), header_ptr, data.len());

//         // buf.advance_mut(data.len());
//         // let header_ptr = header_ptr.add(aligned_header_offset);
//         // write_header_raw::<B, ALIGN>(header_ptr, aligned_data_offset as u32, data.len() as u32)?;

//         // // Write data
//         // let data_ptr = buf_ptr.add(aligned_data_offset);
//         // std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr, data.len());

//         // // Update the buffer's internal write position
//         // let new_len = max(buf_len, total_required_size);
//         // buf.advance_mut(new_len);
//     }

//     Ok(())
// }

// fn write_header_raw<B: ByteOrderExt, const ALIGN: usize>(
//     ptr: *mut u8,
//     data_offset: u32,
//     data_length: u32,
// ) -> Result<(), EncoderError> {
//     unsafe {
//         let mut temp = [0u8; ALIGN];
//         B::write_u32(&mut temp, data_offset);
//         std::ptr::copy_nonoverlapping(temp.as_ptr(), ptr, ALIGN);

//         let mut temp = [0u8; ALIGN];
//         B::write_u32(&mut temp, data_length);
//         std::ptr::copy_nonoverlapping(temp.as_ptr(), ptr.add(ALIGN), ALIGN);
//     }
//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_write_to_existing_buf() {
//         let original = Bytes::from_static(b"Hello, World");
//         let mut buf = BytesMut::new();
//         // let existing_data = &[
//         //     0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
//         //     0, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100,
//         // ];
//         let existing_data = &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
//         buf.extend_from_slice(existing_data);
//         println!("Initial buf: {:?}", &buf.to_vec());

//         let result = write_bytes_unsafe::<BigEndian, 8>(&mut buf, 2, 4, &original);

//         assert!(result.is_ok());

//         println!("Final buf: {:?}", &buf.to_vec());

//         buf.chunk_mut()

//         // let expected = [
//         //     0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0,
//         //     0, 0, 0, 12, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 72, 101, 108, 108,
//         //     111, 44, 32, 87, 111, 114, 108, 100,
//         // ];

//         // assert_eq!(buf.to_vec(), expected);
//     }
// }
