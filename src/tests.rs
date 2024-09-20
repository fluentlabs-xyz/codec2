use alloy_primitives::{Uint, U256};
use alloy_sol_types::{
    sol_data::{self},
    SolType, SolValue,
};
use bytes::{Buf, BufMut, BytesMut};
use hashbrown::HashMap;

use crate::encoder::{SolidityABI, WasmABI};

#[test]
fn test_solidity_abi_u32_encoding() {
    let test_value: u32 = 0x12345678;

    let mut buf = BytesMut::new();
    SolidityABI::<u32>::encode(&test_value, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let alloy_value = sol_data::Uint::<32>::abi_encode(&test_value);

    assert_eq!(encoded, alloy_value);

    let decoded = SolidityABI::<u32>::decode(&&alloy_value[..], 0).unwrap();

    assert_eq!(decoded, test_value);
}

#[test]
fn test_solidity_abi_fixed_array_encoding() {
    let test_value: [u32; 3] = [0x11111111, 0x22222222, 0x33333333];

    let mut buf = BytesMut::new();
    SolidityABI::<[u32; 3]>::encode(&test_value, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let alloy_value = sol_data::FixedArray::<sol_data::Uint<32>, 3>::abi_encode(&test_value);

    assert_eq!(encoded, alloy_value);

    let decoded = SolidityABI::<[u32; 3]>::decode(&&alloy_value[..], 0).unwrap();

    assert_eq!(decoded, test_value);
}

#[test]
fn test_solidity_abi_bytes_encoding() {
    let original = alloy_primitives::Bytes::from_static(b"hello world");

    let mut buf = BytesMut::new();
    SolidityABI::<alloy_primitives::Bytes>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();
    let expected = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 11, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(encoded.to_vec(), expected);

    let sol_encoded = sol_data::Bytes::abi_encode(&original);

    assert_eq!(encoded.to_vec(), sol_encoded);

    let decoded = SolidityABI::<alloy_primitives::Bytes>::decode(&&sol_encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_vec_solidity_abi() {
    let original: Vec<Vec<u32>> = vec![vec![1u32, 2, 3], vec![4, 5], vec![6, 7, 8, 9, 10]];
    let mut buf = BytesMut::new();
    SolidityABI::<Vec<Vec<u32>>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Array<sol_data::Uint<32>>>::abi_encode(&original);

    assert_eq!(hex::encode(encoded), hex::encode(alloy_value));
}

#[test]
fn test_vec_simple() {
    let original: Vec<u32> = vec![1u32, 2, 3, 4, 5];
    let mut buf = BytesMut::new();
    SolidityABI::<Vec<u32>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();
    println!("Encoded Vec: {:?}", hex::encode(&encoded));
    let alloy_value = sol_data::Array::<sol_data::Uint<32>>::abi_encode(&original);
    println!("alloy_value: {:?}", hex::encode(&alloy_value));

    assert_eq!(hex::encode(encoded), hex::encode(alloy_value));
}

// #[test]
// fn test_simple_map() {
//     let mut original = HashMap::new();
//     original.insert(100, 20);
//     original.insert(3, 5);
//     original.insert(1000, 60);

//     let mut buf = BytesMut::new();

//     SolidityABI::<HashMap<u32, u32>>::encode(&original, &mut buf, 0).unwrap();

//     let encoded = buf.freeze();

//     let alloy_encoded =
//         sol_data::Map::<sol_data::Uint<32>, sol_data::Uint<32>>::abi_encode(&original);

//     assert_eq!(hex::encode(encoded), hex::encode(alloy_encoded));

//     let decoded = SolidityABI::<HashMap<u32, u32>>::decode(&&alloy_encoded[..], 0).unwrap();

//     assert_eq!(decoded, original);
// }
