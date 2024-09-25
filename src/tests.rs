use std::vec;

use alloy_primitives::{Uint, U256};
use alloy_sol_types::{
    sol_data::{self},
    SolType, SolValue,
};
use bytes::{Buf, BufMut, BytesMut};
use hashbrown::HashMap;
use hex_literal::hex;

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
    let (offset, length) =
        SolidityABI::<alloy_primitives::Bytes>::partial_decode(&&sol_encoded[..], 0).unwrap();
    println!("Offset: {}, Length: {}", offset, length);
    assert_eq!(offset, 32);
    assert_eq!(length, 11);

    let decoded = SolidityABI::<alloy_primitives::Bytes>::decode(&&sol_encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_vec_solidity_abi_simple() {
    let original: Vec<u32> = vec![1u32, 2, 3];
    let mut buf = BytesMut::new();
    SolidityABI::<Vec<u32>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Uint<32>>::abi_encode(&original);

    // let expected_encoded = hex!(
    //     "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
    // );

    // assert_eq!(encoded.to_vec(), expected_encoded);

    println!("Encoded Vec: {:?}", hex::encode(&encoded));

    assert_eq!(hex::encode(encoded), hex::encode(&alloy_value));

    let decoded = SolidityABI::<Vec<u32>>::decode(&&alloy_value[..], 0).unwrap();
    println!("Decoded Vec: {:?}", decoded);
    assert_eq!(decoded, original);
}
#[test]
fn test_vec_solidity_abi_nested() {
    let original: Vec<Vec<u32>> = vec![vec![1u32, 2, 3], vec![4, 5]];
    let mut buf = BytesMut::new();
    SolidityABI::<Vec<Vec<u32>>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Array<sol_data::Uint<32>>>::abi_encode(&original);

    let expected_encoded = hex!(
        "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000005"
    );

    assert_eq!(encoded.to_vec(), expected_encoded);

    println!("Encoded Vec: {:?}", hex::encode(&encoded));

    let decoded_alloy = sol_data::Array::<sol_data::Array<sol_data::Uint<32>>>::abi_decode(
        &expected_encoded,
        false,
    )
    .unwrap();
    println!("Decoded Vec: {:?}", decoded_alloy);

    assert_eq!(hex::encode(encoded), hex::encode(&alloy_value));

    let decoded = SolidityABI::<Vec<Vec<u32>>>::decode(&&alloy_value[..], 0).unwrap();
    println!("Decoded Vec: {:?}", decoded);
    assert_eq!(decoded, original);
}

#[test]
fn test_vec_wasm_abi() {
    let original: Vec<Vec<u32>> = vec![vec![1u32, 2, 3], vec![4, 5], vec![6, 7, 8, 9, 10]];
    let mut buf = BytesMut::new();
    WasmABI::<Vec<Vec<u32>>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    println!("Encoded Vec: {:?}", hex::encode(&encoded));

    let expected_encoded = hex!("030000000c0000004c00000003000000240000000c0000000200000030000000080000000500000038000000140000000100000002000000030000000400000005000000060000000700000008000000090000000a000000");

    assert_eq!(encoded.to_vec(), expected_encoded);

    let decoded = WasmABI::<Vec<Vec<u32>>>::decode(&&encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_empty_vector() {
    let empty_vec: Vec<u32> = vec![];

    let mut buf = BytesMut::new();
    SolidityABI::<Vec<u32>>::encode(&empty_vec, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Uint<32>>::abi_encode(&empty_vec);

    assert_eq!(encoded, alloy_value);

    let decoded = SolidityABI::<Vec<u32>>::decode(&&alloy_value[..], 0).unwrap();
    assert_eq!(decoded, empty_vec);
}
#[test]
fn test_empty_bytes_solidity() {
    let original: alloy_primitives::Bytes = alloy_primitives::Bytes::new();
    let mut buf = BytesMut::new();

    SolidityABI::<alloy_primitives::Bytes>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    println!("Encoded Bytes: {:?}", hex::encode(&encoded));

    let alloy_encoded = sol_data::Bytes::abi_encode(&original);

    assert_eq!(encoded, alloy_encoded);

    let decoded = SolidityABI::<alloy_primitives::Bytes>::decode(&&alloy_encoded[..], 0).unwrap();
    println!("Decoded Bytes: {:?}", decoded.to_vec());

    assert_eq!(original, decoded);
}

#[test]
fn test_vec_partial_decoding_solidity() {
    let original: Vec<u32> = vec![1u32, 2, 3, 4, 5];
    let mut buf = BytesMut::new();
    SolidityABI::<Vec<u32>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Uint<32>>::abi_encode(&original);

    println!("Encoded Vec: {:?}", hex::encode(&encoded));

    assert_eq!(hex::encode(encoded), hex::encode(&alloy_value));

    // offset, length
    let decoded_header = SolidityABI::<Vec<u32>>::partial_decode(&&alloy_value[..], 0).unwrap();

    assert_eq!(decoded_header, (32, 5));
}

#[test]
fn test_vec_partial_decoding_wasm() {
    let original: Vec<u32> = vec![1u32, 2, 3, 4, 5];
    let mut buf = BytesMut::new();
    WasmABI::<Vec<u32>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    println!("Encoded Vec: {:?}", hex::encode(&encoded));

    // offset, length
    let decoded_header = WasmABI::<Vec<u32>>::partial_decode(&&encoded[..], 0).unwrap();

    assert_eq!(decoded_header, (12, 20));

    assert_eq!(encoded.chunk()[12..20], vec![1, 0, 0, 0, 2, 0, 0, 0]);
}

#[test]
fn test_address_encoding() {
    let original =
        alloy_primitives::Address::from(hex!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"));

    let mut buf = BytesMut::new();
    SolidityABI::<alloy_primitives::Address>::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let alloy_encoded = sol_data::Address::abi_encode(&original);

    assert_eq!(encoded.to_vec(), alloy_encoded);

    let decoded = SolidityABI::<alloy_primitives::Address>::decode(&&alloy_encoded[..], 0).unwrap();

    let alloy_decoded = sol_data::Address::abi_decode(&alloy_encoded, false).unwrap();

    assert_eq!(decoded, alloy_decoded);
    assert_eq!(decoded, original);
}
#[test]
fn test_simple_map_wasm_abi() {
    let mut original = HashMap::new();
    original.insert(100, 20);
    original.insert(3, 5);
    original.insert(1000, 60);

    let mut buf = BytesMut::new();

    WasmABI::<HashMap<u32, u32>>::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let expected_encoded = hex!(
        "03000000140000000c000000200000000c0000000300000064000000e803000005000000140000003c000000"
    );

    assert_eq!(encoded.to_vec(), expected_encoded);

    println!("Encoded Map: {:?}", hex::encode(&encoded));

    // let decoded = WasmABI::<HashMap<u32, u32>>::decode(&&encoded[..], 0).unwrap();

    // assert_eq!(decoded, original);
}
