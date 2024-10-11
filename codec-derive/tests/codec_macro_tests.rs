// use byteorder::ByteOrder;
// use bytes::{Buf, BytesMut};
// use codec2::{
//     encoder::{align_up, Encoder},
//     error::CodecError,
// };
// use codec_derive::Codec;
// use std::panic::catch_unwind;

// #[derive(Codec)]
// struct TestStruct {
//     field1: u32,
//     field2: Vec<u8>,
// }

// #[test]
// fn test_codec_macro() {
//     let test_struct = TestStruct {
//         field1: 42,
//         field2: vec![1, 2, 3, 4],
//     };

//     // Capture the output of eprintln!
//     let result = catch_unwind(|| {
//         let mut buf = BytesMut::new();
//         // test_struct.encode(&mut buf, 0).unwrap();
//     });

//     // Check if the output contains the expected debug information
//     assert!(result.is_ok());
// }
