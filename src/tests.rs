use std::vec;

use alloy_primitives::Bytes;
use bytes::{buf, Buf, BufMut, BytesMut};
use hashbrown::{HashMap, HashSet};

use alloy_sol_types::{sol_data::*, SolType, SolValue};

#[test]
fn test_byteorder() {}
// type MyU32 = u32;

// type MyVec = Vec<u32>;

// impl Encoder<MyU32> for MyU32 {
//     const HEADER_SIZE: usize = core::mem::size_of::<u32>();

//     fn encode<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, field_offset: usize) {
//         let aligned_offset = A::align(field_offset);
//         let aligned_size = A::align(Self::HEADER_SIZE);
//         let total_size = aligned_offset + aligned_size;

//         if buffer.len() < total_size {
//             buffer.resize(total_size, 0);
//         }

//         // Заполняем выравнивание нулями
//         buffer[field_offset..aligned_offset].fill(0);

//         // Используем новый метод write из трейта Endian
//         E::write::<u32>(
//             &mut buffer[aligned_offset..aligned_offset + aligned_size],
//             *self,
//         );

//         // Заполняем оставшееся пространство нулями
//         buffer[aligned_offset + aligned_size..total_size].fill(0);
//         println!("value = {:?}", self);
//         println!("buffer = {:?}", buffer.to_vec());
//     }

//     fn decode_header<A: Alignment, E: Endian>(
//         bytes: &bytes::Bytes,
//         field_offset: usize,
//         result: &mut MyU32,
//     ) -> (usize, usize) {
//         let aligned_offset = A::align(field_offset);
//         let aligned_size = A::align(Self::HEADER_SIZE);

//         if bytes.len() < aligned_offset + aligned_size {
//             return (0, 0);
//         }

//         *result = E::read::<u32>(&bytes[aligned_offset..aligned_offset + Self::HEADER_SIZE]);

//         (0, A::SIZE.max(Self::HEADER_SIZE))
//     }

//     fn decode_body<A: Alignment, E: Endian>(
//         bytes: &bytes::Bytes,
//         field_offset: usize,
//         result: &mut MyU32,
//     ) {
//         Self::decode_header::<A, E>(bytes, field_offset, result);
//     }
// }

// #[test]
// fn test_header_aligment() {
//     // let mut buffer = BytesMut::new();
//     // let values: Vec<MyU32> = vec![1, 2, 3];
//     // values.encode::<Align32, BigEndian>(&mut buffer, 0);

//     // let encoded = buffer.freeze();
//     // println!(">>>{:?}", hex::encode(&encoded));

//     // let expected = "000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003";
//     // assert_eq!(
//     //     hex::encode(&encoded),
//     //     expected,
//     //     "Encoded value is not equal to expected"
//     // );

//     // let values: Vec<MyU32> = vec![1, 2, 3];

//     // let mut buf = BytesMut::new();

//     // values.encode_abi::<Align32, BigEndian>(&mut buf, 0);

//     // let encoded = buf.freeze();

//     // println!("1<<<{}", hex::encode(&encoded));

//     let values: Vec<MyU32> = vec![1, 2, 3];

//     let encoded = MyVec::abi_encode(&values);

//     println!("<<<{}", hex::encode(&encoded));

//     let expected = "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003";

//     let mut buf = BytesMut::new();
//     values.encode_abi::<Align32, BigEndian>(&mut buf, 0);

//     let encoded = buf.freeze();

//     println!("2<<<{}", hex::encode(&encoded));

//     // let tokens: Vec<Token> = values.into_iter().map(|v| Token::Uint(v.into())).collect();
//     // let array_token = Token::Array(tokens);
//     // let encoded = encode(&[array_token]);

//     // println!("<<<{}", hex::encode(&encoded));
// }
