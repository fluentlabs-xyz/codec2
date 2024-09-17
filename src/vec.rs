// extern crate alloc;
// use crate::{
//     encoder::{align_offset, ByteOrderExt, Encoder, EncoderError},
//     evm::{read_bytes, write_bytes},
// };
// use alloc::vec::Vec;
// use bytes::{Buf, BufMut, Bytes, BytesMut};
// vec![vec![1], vec![2, 3], vec![4, 5, 6]];
// 0..4  03000000 - 3 элемента в массиве
// 4..8  0c000000 - 12 - офсет для данных
// 8..12 3c000000 - 60 - длина данных
// ----------- реальные данные
// 01000000 - 1 длина первого массива
// 24000000 - 36 - офсет для данных
// 04000000 - 4 - размер данных в байтах
// 02000000 - длина второго массива
// 28000000 - 40 - офсет для данных
// 08000000 - 8 - размер данных в байтах
// 03000000 - 3 длина третьего массива
// 30000000 - 48 - офсет для данных
// 0c000000 - 12 - размер данных в байтах
// 010000000200000003000000040000000500000006000000
// ///
// /// We encode dynamic arrays as following:
// /// - header
// /// - + length - number of elements inside vector
// /// - + offset - offset inside structure
// /// - + size - number of encoded bytes
// /// - body
// /// - + raw bytes of the vector
// ///
// /// We don't encode empty vectors, instead we store 0 as length,
// /// it helps to reduce empty vector size from 12 to 4 bytes.
// impl<T> Encoder for Vec<T>
// where
//     T: Sized + Encoder + Default,
// {
//     const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;
//     const DATA_SIZE: usize = 0; // Динамический размер

//     fn encode<B: ByteOrderExt, const ALIGN: usize>(
//         &self,
//         buf: &mut impl BufMut,
//         offset: usize,
//     ) -> Result<(), EncoderError> {
//         // let aligned_offset = A::align(offset);
//         // let elem_size = A::align(4);

//         // if buffer.len() < aligned_offset + elem_size {
//         //     buffer.resize(aligned_offset + elem_size, 0);
//         // }
//         let aligned_offset = align_offset::<ALIGN>(offset);
//         let aligned_elem_size = align_offset::<ALIGN>(4);
//         let aligned_header_size = align_offset::<ALIGN>(Self::HEADER_SIZE);

//         if buf.remaining_mut() < aligned_offset + aligned_elem_size {
//             return Err(EncoderError::BufferTooSmall {
//                 required: aligned_offset + aligned_elem_size,
//                 available: buf.remaining_mut(),
//                 msg: "failed to encode vector".to_string(),
//             });
//         };

//         let mut header = [0u8; 12];
//         B::write_u32(&mut header[0..4], self.len() as u32);

//         // If vector is empty, we don't need to encode anything
//         if self.is_empty() {
//             B::write_u32(&mut header[4..8], 0); // offset
//             B::write_u32(&mut header[8..12], 0); // size
//             buf.put_bytes(0, aligned_offset);
//             buf.put_slice(&header);
//             return Ok(());
//         }
//         // buf.advance_mut(elem_size);)

//         // Кодируем элементы во временный буфер
//         let mut temp_buf = BytesMut::new();
//         for item in self {
//             item.encode::<B, ALIGN>(&mut temp_buf, temp_buf.len())?;
//         }

//              // // Vector size
//         // E::write::<u32>(
//         //     &mut buffer[aligned_offset..aligned_offset + elem_size],
//         //     self.len() as u32,
//         // );
//         // Записываем смещение и размер данных
//         B::write_u32(
//             &mut header[4..8],
//             (aligned_offset + aligned_header_size) as u32,
//         );
//         B::write_u32(&mut header[8..12], temp_buf.len() as u32);

//         // Записываем заголовок и данные
//         buf.put_bytes(0, aligned_offset);
//         buf.put_slice(&header);
//         buf.put(temp_buf);

//         Ok(())

//         // // Vector size
//         // E::write::<u32>(
//         //     &mut buffer[aligned_offset..aligned_offset + elem_size],
//         //     self.len() as u32,
//         // );

//         // // encode values
//         // // reserve space for headers
//         // let mut value_encoder = BytesMut::zeroed(A::SIZE.max(T::HEADER_SIZE) * self.len());

//         // for (index, obj) in self.iter().enumerate() {
//         //     let elem_offset = A::SIZE.max(T::HEADER_SIZE) * index;
//         //     obj.encode::<A, E>(&mut value_encoder, elem_offset);
//         // }

//         // write_bytes::<A, E>(buffer, aligned_offset + 4, &value_encoder.freeze());
//     }

//     fn decode<B: ByteOrderExt, const ALIGN: usize>(
//         buf: &mut impl Buf,
//         offset: usize,
//     ) -> Result<Self, EncoderError> {
//         let aligned_offset = A::align(offset);
//         let elem_size = A::align(4);
//         let data_len = E::read::<u32>(&bytes[aligned_offset..aligned_offset + elem_size]) as usize;

//         if data_len == 0 {
//             result.clear();
//             return;
//         }

//         let input_bytes = read_bytes::<A, E>(bytes, aligned_offset + elem_size);

//         let elem_size = A::SIZE.max(T::HEADER_SIZE);
//         *result = (0..data_len)
//             .map(|i| {
//                 let mut value = T::default();
//                 let elem_offset = elem_size * i;
//                 T::decode_body::<A, E>(&input_bytes, elem_offset, &mut value);
//                 value
//             })
//             .collect();
//     }

//     fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
//         buf: &mut impl Buf,
//         offset: usize,
//     ) -> Result<Self, EncoderError> {
//         let aligned_offset = A::align(field_offset);
//         let elem_size = A::align(4);

//         // TODO: d1r1 maybe we should return an error here?
//         if bytes.remaining() < aligned_offset + elem_size {
//             return (0, 0);
//         }

//         // Vector size
//         let count = E::read::<u32>(&bytes[aligned_offset..aligned_offset + elem_size]) as usize;

//         // If vector is empty, we don't need to decode anything
//         if count == 0 {
//             result.clear();
//             return (0, 0);
//         }

//         // Get data offset and length
//         let data_offset =
//             E::read::<u32>(&bytes[aligned_offset + elem_size..aligned_offset + elem_size * 2])
//                 as usize;
//         let data_length =
//             E::read::<u32>(&bytes[aligned_offset + elem_size * 2..aligned_offset + elem_size * 3])
//                 as usize;

//         result.reserve(data_length);

//         (data_offset, data_length)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::encoder::{Align0, Align4, Align8, BigEndian, Encoder, LittleEndian};

//     #[test]
//     fn test_empty_vec_u32() {
//         let original: Vec<u32> = Vec::new();
//         let mut buffer = BytesMut::new();

//         original.encode::<Align4, LittleEndian>(&mut buffer, 0);
//         let encoded = buffer.freeze();
//         let expected = hex::decode("000000000c00000000000000").expect("Failed to decode hex");
//         assert_eq!(encoded, Bytes::from(expected));

//         let mut decoded: Vec<u32> = Vec::new();
//         Vec::<u32>::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

//         assert_eq!(original, decoded);
//     }

//     #[test]
//     fn test_vec_u32() {
//         let original: Vec<u32> = vec![1, 2, 3, 4];
//         let mut buffer = BytesMut::new();

//         original.encode::<Align4, BigEndian>(&mut buffer, 0);
//         let encoded = buffer.freeze();

//         let mut decoded: Vec<u32> = Vec::new();
//         Vec::<u32>::decode_body::<Align4, BigEndian>(&encoded, 0, &mut decoded);

//         assert_eq!(original, decoded);
//     }

//     #[test]
//     fn test_vec_u32_with_offset() {
//         let original: Vec<u32> = vec![1, 2, 3, 4, 5];
//         let mut buffer = BytesMut::new();
//         buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

//         original.encode::<Align4, LittleEndian>(&mut buffer, 3);
//         let encoded = buffer.freeze();
//         println!("{:?}", hex::encode(&encoded));

//         let mut decoded: Vec<u32> = Vec::new();
//         Vec::<u32>::decode_body::<Align4, LittleEndian>(&encoded, 3, &mut decoded);

//         assert_eq!(original, decoded);
//     }
//     #[test]
//     fn test_vec_u8_with_offset() {
//         let original: Vec<u8> = vec![1, 2, 3, 4, 5];
//         let mut buffer = BytesMut::new();
//         buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

//         original.encode::<Align4, LittleEndian>(&mut buffer, 3);
//         let encoded = buffer.freeze();
//         println!("{:?}", hex::encode(&encoded));

//         let mut decoded: Vec<u8> = Vec::new();
//         Vec::<u8>::decode_body::<Align4, LittleEndian>(&encoded, 3, &mut decoded);

//         assert_eq!(original, decoded);
//     }

//     #[test]
//     fn test_nested_vec() {
//         let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

//         let mut buffer = BytesMut::new();
//         original.encode::<Align0, LittleEndian>(&mut buffer, 0);
//         let encoded = buffer.freeze();
//         println!("{:?}", hex::encode(&encoded));
//         let expected_encoded = "020000000c00000022000000020000001800000004000000030000001c0000000600000003000400050006000700";

//         assert_eq!(hex::encode(&encoded), expected_encoded);

//         let mut decoded: Vec<Vec<u16>> = Vec::new();
//         Vec::<Vec<u16>>::decode_body::<Align0, LittleEndian>(&encoded, 0, &mut decoded);

//         assert_eq!(original, decoded);
//     }
//     #[test]
//     fn test_nested_vec_a4_le() {
//         let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

//         let mut buffer = BytesMut::new();
//         original.encode::<Align4, LittleEndian>(&mut buffer, 0);
//         let encoded = buffer.freeze();
//         let mut decoded: Vec<Vec<u16>> = Vec::new();
//         Vec::<Vec<u16>>::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

//         assert_eq!(original, decoded);
//     }
//     #[test]
//     fn test_nested_vec_a4_be() {
//         let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

//         let mut buffer = BytesMut::new();
//         original.encode::<Align4, BigEndian>(&mut buffer, 0);
//         let encoded = buffer.freeze();

//         let mut decoded: Vec<Vec<u16>> = Vec::new();
//         Vec::<Vec<u16>>::decode_body::<Align4, BigEndian>(&encoded, 0, &mut decoded);

//         assert_eq!(original, decoded);
//     }

//     #[test]
//     fn test_large_vec() {
//         let original: Vec<u64> = (0..1000).collect();
//         let mut buffer = BytesMut::new();

//         original.encode::<Align8, BigEndian>(&mut buffer, 0);
//         let encoded = buffer.freeze();

//         let mut decoded: Vec<u64> = Vec::new();
//         Vec::<u64>::decode_body::<Align8, BigEndian>(&encoded, 0, &mut decoded);

//         assert_eq!(original, decoded);
//     }
// }
