extern crate alloc;
use crate::{
    empty::EmptyVec,
    encoder::{
        align_up,
        is_big_endian,
        read_u32_aligned,
        write_u32_aligned,
        Encoder,
        SolidityABI,
        WasmABI,
    },
    error::CodecError,
};
use alloc::vec;
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{
    sol,
    sol_data::{self},
    SolType,
    SolValue,
};
use byteorder::{BigEndian, ByteOrder, BE, LE};
use bytes::{Buf, BufMut, BytesMut};
use codec_derive::Codec;
use core::str;
use hashbrown::HashMap;
use hex_literal::hex;
use pretty_hex::*;

pub fn print_bytes<B: ByteOrder, const ALIGN: usize>(buf: &[u8]) {
    for (i, chunk) in buf.chunks(ALIGN).enumerate() {
        let offset = i * ALIGN;
        print!("{:04x}: ", offset);

        if is_big_endian::<B>() {
            for &byte in &chunk[&chunk.len() - 4..] {
                print!("{:02x} ", byte);
            }
        } else {
            for &byte in &chunk[..4] {
                print!("{:02x} ", byte);
            }
        }

        for _ in chunk.len()..ALIGN {
            print!("   ");
        }
        print!("  ||  {:03}", offset);
        let decimal_value = read_u32_aligned::<B, ALIGN>(&chunk, 0).unwrap();
        println!(": {:03} |", decimal_value);
    }
}

#[derive(Default, Debug, PartialEq)]
struct TestStruct {
    bool_val: bool,
    u8_val: u8,
    uint_val: (u16, u32, u64),
    int_val: (i16, i32, i64),
    u256_val: alloy_primitives::U256,
    address_val: alloy_primitives::Address,
    bytes_val: Bytes,
    vec_val: Vec<u32>,
}

impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for TestStruct {
    const HEADER_SIZE: usize = 32; // Оффсет 32 байта для динамических структур
    const IS_DYNAMIC: bool = true;

    fn encode(&self, buf: &mut BytesMut, mut offset: usize) -> Result<(), CodecError> {
        write_u32_aligned::<B, ALIGN>(buf, offset, 32u32);
        offset += align_up::<ALIGN>(4);

        let mut data_offset = offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        // Кодируем статические поля
        <bool as Encoder<B, ALIGN, false>>::encode(&self.bool_val, buf, offset)?;
        offset += <bool as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        <u8 as Encoder<B, ALIGN, false>>::encode(&self.u8_val, buf, offset)?;
        offset += <u8 as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        <(u16, u32, u64) as Encoder<B, ALIGN, false>>::encode(&self.uint_val, buf, offset)?;
        offset += <(u16, u32, u64) as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        <(i16, i32, i64) as Encoder<B, ALIGN, false>>::encode(&self.int_val, buf, offset)?;
        offset += <(i16, i32, i64) as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        <U256 as Encoder<B, ALIGN, false>>::encode(&self.u256_val, buf, offset)?;
        offset += <U256 as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        <Address as Encoder<B, ALIGN, false>>::encode(&self.address_val, buf, offset)?;
        offset += <Address as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        // Кодируем динамические поля
        write_u32_aligned::<B, ALIGN>(buf, offset, data_offset as u32);
        <Bytes as Encoder<B, ALIGN, false>>::encode(&self.bytes_val, buf, data_offset)?;
        data_offset += <Bytes as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        offset += align_up::<ALIGN>(4);

        write_u32_aligned::<B, ALIGN>(buf, offset, data_offset as u32);
        <Vec<u32> as Encoder<B, ALIGN, false>>::encode(&self.vec_val, buf, data_offset)?;
        data_offset += <Vec<u32> as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        offset += align_up::<ALIGN>(4);

        Ok(())
    }

    fn decode(buf: &impl Buf, mut offset: usize) -> Result<Self, CodecError> {
        // Пропускаем начальный оффсет
        offset += align_up::<ALIGN>(4);

        let bool_val = <bool as Encoder<B, ALIGN, false>>::decode(buf, offset)?;
        offset += <bool as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        let u8_val = <u8 as Encoder<B, ALIGN, false>>::decode(buf, offset)?;
        offset += <u8 as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        let uint_val = <(u16, u32, u64) as Encoder<B, ALIGN, false>>::decode(buf, offset)?;
        offset += <(u16, u32, u64) as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        let int_val = <(i16, i32, i64) as Encoder<B, ALIGN, false>>::decode(buf, offset)?;
        offset += <(i16, i32, i64) as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        let u256_val = <U256 as Encoder<B, ALIGN, false>>::decode(buf, offset)?;
        offset += <U256 as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        let address_val = <Address as Encoder<B, ALIGN, false>>::decode(buf, offset)?;
        offset += <Address as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        let bytes_offset = read_u32_aligned::<B, ALIGN>(buf, offset)? as usize;
        let bytes_val = <Bytes as Encoder<B, ALIGN, false>>::decode(buf, bytes_offset)?;
        offset += align_up::<ALIGN>(4);

        let vec_offset = read_u32_aligned::<B, ALIGN>(buf, offset)? as usize;
        let vec_val = <Vec<u32> as Encoder<B, ALIGN, false>>::decode(buf, vec_offset)?;

        Ok(TestStruct {
            bool_val,
            u8_val,
            uint_val,
            int_val,
            u256_val,
            address_val,
            bytes_val,
            vec_val,
        })
    }

    fn partial_decode(buf: &impl Buf, mut offset: usize) -> Result<(usize, usize), CodecError> {
        let mut total_length = <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE;

        // Пропускаем начальный оффсет
        offset += align_up::<ALIGN>(4);

        let (_, length) = <bool as Encoder<B, ALIGN, false>>::partial_decode(buf, offset)?;
        offset += <bool as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        total_length += length;

        let (_, length) = <u8 as Encoder<B, ALIGN, false>>::partial_decode(buf, offset)?;
        offset += <u8 as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        total_length += length;

        let (_, length) =
            <(u16, u32, u64) as Encoder<B, ALIGN, false>>::partial_decode(buf, offset)?;
        offset += <(u16, u32, u64) as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        total_length += length;

        let (_, length) =
            <(i16, i32, i64) as Encoder<B, ALIGN, false>>::partial_decode(buf, offset)?;
        offset += <(i16, i32, i64) as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        total_length += length;

        let (_, length) = <U256 as Encoder<B, ALIGN, false>>::partial_decode(buf, offset)?;
        offset += <U256 as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        total_length += length;

        let (_, length) = <Address as Encoder<B, ALIGN, false>>::partial_decode(buf, offset)?;
        offset += <Address as Encoder<B, ALIGN, false>>::HEADER_SIZE;
        total_length += length;

        let bytes_offset = read_u32_aligned::<B, ALIGN>(buf, offset)? as usize;
        let (_, length) = <Bytes as Encoder<B, ALIGN, false>>::partial_decode(buf, bytes_offset)?;
        offset += align_up::<ALIGN>(4);
        total_length += length;

        let vec_offset = read_u32_aligned::<B, ALIGN>(buf, offset)? as usize;
        let (_, length) = <Vec<u32> as Encoder<B, ALIGN, false>>::partial_decode(buf, vec_offset)?;
        total_length += length;

        Ok((offset, total_length))
    }
}
impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for TestStruct {
    const HEADER_SIZE: usize = 32;
    const IS_DYNAMIC: bool = true;

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        println!("xxxxx");
        let aligned_offset = align_up::<ALIGN>(offset);
        let mut current_offset = aligned_offset;
        let mut tmp = BytesMut::new();

        // Кодируем статические поля
        <bool as Encoder<B, ALIGN, true>>::encode(&self.bool_val, &mut tmp, current_offset)?;

        current_offset += align_up::<ALIGN>(<bool as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        <u8 as Encoder<B, ALIGN, true>>::encode(&self.u8_val, &mut tmp, current_offset)?;

        current_offset += align_up::<ALIGN>(<u8 as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        println!("current offset: {:?}", current_offset);

        <(u16, u32, u64) as Encoder<B, ALIGN, true>>::encode(
            &self.uint_val,
            &mut tmp,
            current_offset,
        )?;
        current_offset +=
            align_up::<ALIGN>(<(u16, u32, u64) as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        <(i16, i32, i64) as Encoder<B, ALIGN, true>>::encode(
            &self.int_val,
            &mut tmp,
            current_offset,
        )?;
        current_offset +=
            align_up::<ALIGN>(<(i16, i32, i64) as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        <U256 as Encoder<B, ALIGN, true>>::encode(&self.u256_val, &mut tmp, current_offset)?;
        current_offset += align_up::<ALIGN>(<U256 as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        <Address as Encoder<B, ALIGN, true>>::encode(&self.address_val, &mut tmp, current_offset)?;
        current_offset += align_up::<ALIGN>(<Address as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        if tmp.len() < current_offset + 64 {
            // 64 - 32 + 32
            tmp.resize(current_offset + 64, 0);
        }

        println!("tmp.len(): {:?}", tmp.len());
        println!("current_offset: {:?}", current_offset);

        let bytes_offset = tmp.len();
        <Bytes as Encoder<B, ALIGN, true>>::encode(&self.bytes_val, &mut tmp, 320)?;

        let vec_offset = tmp.len();

        <Vec<u32> as Encoder<B, ALIGN, true>>::encode(&self.vec_val, &mut tmp, 352)?;

        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 32);
        // println!("buf: {:?}", &buf.chunk()[..]);

        buf.extend_from_slice(&tmp);

        Ok(())
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let mut current_offset = align_up::<ALIGN>(offset);

        // skip first dynamic offset
        let mut tmp = &buf.chunk()[32..];

        println!("1. current_offset: {:?}", current_offset);
        let bool_val = <bool as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset += align_up::<ALIGN>(<bool as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        println!("2. current_offset: {:?}", current_offset);
        println!("bool_val: {:?}", bool_val);

        let u8_val = <u8 as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset += align_up::<ALIGN>(<u8 as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        let uint_val =
            <(u16, u32, u64) as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset +=
            align_up::<ALIGN>(<(u16, u32, u64) as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        println!("uint_val: {:?}", uint_val);

        let int_val =
            <(i16, i32, i64) as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset +=
            align_up::<ALIGN>(<(i16, i32, i64) as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        let u256_val = <U256 as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset += align_up::<ALIGN>(<U256 as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        let address_val = <Address as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset += align_up::<ALIGN>(<Address as Encoder<B, ALIGN, true>>::HEADER_SIZE);

        println!("address_val: {:?}", address_val);

        // Decode dynamic fields
        println!(">>>current_offset: {:?}", current_offset);
        let bytes_val = <Bytes as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;
        current_offset += 32;

        println!("bytes_val: {:?}", bytes_val);

        // println!(">>>current_offset: {:?}", current_offset);
        // // println!("buf: {:?}", &buf.chunk()[..]);
        let vec_val = <Vec<u32> as Encoder<B, ALIGN, true>>::decode(&mut tmp, current_offset)?;

        println!("vec_val: {:?}", vec_val);
        Ok(TestStruct {
            bool_val,
            u8_val,
            uint_val,
            int_val,
            u256_val,
            address_val,
            bytes_val,
            vec_val,
        })
    }

    fn partial_decode(buf: &impl Buf, mut offset: usize) -> Result<(usize, usize), CodecError> {
        let mut total_length = 0;

        let (_, length) = <bool as Encoder<B, ALIGN, true>>::partial_decode(buf, offset)?;
        offset += 32;
        total_length += length;

        let (_, length) = <u8 as Encoder<B, ALIGN, true>>::partial_decode(buf, offset)?;
        offset += 32;
        total_length += length;

        let (_, length) =
            <(u16, u32, u64) as Encoder<B, ALIGN, true>>::partial_decode(buf, offset)?;
        offset += 3 * 32;
        total_length += length;

        let (_, length) =
            <(i16, i32, i64) as Encoder<B, ALIGN, true>>::partial_decode(buf, offset)?;
        offset += 3 * 32;
        total_length += length;

        let (_, length) = <U256 as Encoder<B, ALIGN, true>>::partial_decode(buf, offset)?;
        offset += 32;
        total_length += length;

        let (_, length) = <Address as Encoder<B, ALIGN, true>>::partial_decode(buf, offset)?;
        total_length += length;

        Ok((offset, total_length))
    }
}

#[test]
fn test_struct_encoding_sol() {
    // Create an instance of TestStruct
    let test_struct = TestStruct {
        bool_val: true,
        u8_val: 42,
        uint_val: (1000, 1_000_000, 1_000_000_000),
        int_val: (-1000, -1_000_000, -1_000_000_000),
        u256_val: U256::from(12345),
        address_val: Address::repeat_byte(0xAA),
        bytes_val: Bytes::from(vec![1, 2, 3, 4, 5]),
        vec_val: vec![10, 20, 30],
    };

    // Encode using our Encoder
    let mut buf = BytesMut::new();
    SolidityABI::encode(&test_struct, &mut buf, 0).unwrap();
    let encoded = buf.freeze();
    println!("Our codec Encoded:");
    println!("{:?}", hex::encode(&encoded));

    print_bytes::<BE, 32>(&encoded);

    // Create an equivalent structure in alloy_sol_types
    sol! {
        struct TestStructSol {
            bool bool_val;
            uint8 u8_val;
            (uint16, uint32, uint64) uint_val;
            (int16, int32, int64) int_val;
            uint256 u256_val;
            address address_val;
            bytes bytes_val;
            uint32[] vec_val;
        }
    }

    let test_struct_sol = TestStructSol {
        bool_val: true,
        u8_val: 42,
        uint_val: (1000, 1_000_000, 1_000_000_000),
        int_val: (-1000, -1_000_000, -1_000_000_000),
        u256_val: U256::from(12345),
        address_val: Address::repeat_byte(0xAA),
        bytes_val: Bytes::from(vec![1, 2, 3, 4, 5]),
        vec_val: vec![10, 20, 30],
    };

    let alloy_encoded = &test_struct_sol.abi_encode();
    println!("Alloy Encoded:");
    println!("{:?}", hex::encode(&alloy_encoded));
    print_bytes::<BE, 32>(&alloy_encoded);

    // Compare the results
    assert_eq!(
        hex::encode(&encoded),
        hex::encode(alloy_encoded),
        "Encoding mismatch between our Encoder and alloy_sol_types"
    );

    // Additionally, we can test decoding
    let decoded = SolidityABI::<TestStruct>::decode(&encoded, 0).unwrap();
    assert_eq!(decoded, test_struct, "Decoding mismatch");
}

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
fn test_i32_sol() {
    let test_value: i32 = -0x12345678;
    let mut buf = BytesMut::new();
    SolidityABI::<i32>::encode(&test_value, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Int::<32>::abi_encode(&test_value);

    let cfg = HexConfig {
        title: false,
        width: 8,
        group: 0,
        ..HexConfig::default()
    };

    println!(">>>>: {}", config_hex(&encoded, cfg));

    // println!("Encoded i32: {:?}", hex::encode(&encoded));

    assert_eq!(hex::encode(&encoded), hex::encode(&alloy_value));
}

#[test]
fn test_u32_sol() {
    let test_value: u32 = 0x12345678;
    let mut buf = BytesMut::new();
    SolidityABI::<u32>::encode(&test_value, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let alloy_value = sol_data::Uint::<32>::abi_encode(&test_value);

    println!("Encoded u32: {:?}", hex::encode(&encoded));

    assert_eq!(hex::encode(&encoded), hex::encode(&alloy_value));
}

#[test]
fn test_solidity_abi_fixed_array_encoding() {
    let test_value: [u32; 3] = [0x11111111, 0x22222222, 0x33333333];

    let mut buf = BytesMut::new();
    SolidityABI::encode(&test_value, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let alloy_value = sol_data::FixedArray::<sol_data::Uint<32>, 3>::abi_encode(&test_value);

    println!("alloy encoded: {:?}", alloy_value);

    assert_eq!(encoded, alloy_value);

    let decoded = SolidityABI::<[u32; 3]>::decode(&&alloy_value[..], 0).unwrap();

    assert_eq!(decoded, test_value);
}

#[test]
fn test_solidity_abi_bytes_encoding() {
    let original = alloy_primitives::Bytes::from_static(b"hello world");

    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();
    let expected = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 11, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(encoded.to_vec(), expected);

    // let sol_encoded = sol_data::Bytes::abi_encode(&original);

    // assert_eq!(encoded.to_vec(), sol_encoded);

    // let (offset, length) =
    //     SolidityABI::<alloy_primitives::Bytes>::partial_decode(&&sol_encoded[..], 0).unwrap();
    // assert_eq!(offset, 64);
    // assert_eq!(length, 11);

    // let decoded = SolidityABI::<alloy_primitives::Bytes>::decode(&&sol_encoded[..], 0).unwrap();

    // let alloy_decoded = sol_data::Bytes::abi_decode(&sol_encoded, false).unwrap();

    // println!("Decoded Bytes (our): {:?}", decoded.to_vec());
    // println!("Decoded Bytes (alloy): {:?}", alloy_decoded.to_vec());

    // assert_eq!(decoded, original);
}

#[test]
fn test_solidity_abi_fixed_bytes_encoding() {
    // Use FixedBytes<11> to match the length of "hello world"
    let original = alloy_primitives::FixedBytes::<11>::from_slice(b"hello world");

    let mut buf = BytesMut::new();
    SolidityABI::<alloy_primitives::FixedBytes<11>>::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    // FixedBytes are encoded inline without length prefix
    let expected = [
        104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(encoded.to_vec(), expected);

    // Encode using sol_data for comparison
    let sol_encoded = sol_data::FixedBytes::<11>::abi_encode(&original);

    assert_eq!(encoded.to_vec(), sol_encoded);

    // FixedBytes don't have a separate offset and length in their encoding
    let (offset, length) =
        SolidityABI::<alloy_primitives::FixedBytes<11>>::partial_decode(&&sol_encoded[..], 0)
            .unwrap();
    println!("Offset: {}, Length: {}", offset, length);
    assert_eq!(offset, 0); // FixedBytes are encoded inline
    assert_eq!(length, 32); // Always padded to 32 bytes

    let decoded =
        SolidityABI::<alloy_primitives::FixedBytes<11>>::decode(&&sol_encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
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
fn test_vec_solidity_abi_simple() {
    let original: Vec<u32> = vec![1, 2, 3];
    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Uint<32>>::abi_encode(&original);

    let expected_encoded = hex!(
        "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
    );

    assert_eq!(encoded.to_vec(), expected_encoded);

    println!("Encoded Vec: {:?}", hex::encode(&encoded));

    assert_eq!(hex::encode(encoded), hex::encode(&alloy_value));

    let decoded = SolidityABI::<Vec<u32>>::decode(&&alloy_value[..], 0).unwrap();
    println!("Decoded Vec: {:?}", decoded);
    assert_eq!(decoded, original);
}

#[test]
fn test_vec_solidity_abi_nested() {
    let original: Vec<Vec<u32>> = vec![vec![1, 2, 3], vec![4, 5]];
    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();
    let encoded = buf.freeze();

    let alloy_value = sol_data::Array::<sol_data::Array<sol_data::Uint<32>>>::abi_encode(&original);

    let expected_encoded = hex!(
        "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000005"
    );
    println!("codec2");
    print_bytes::<BE, 32>(&encoded);
    println!("solidity");
    print_bytes::<BE, 32>(&alloy_value);
    // println!("Encoded Vec: {:?}", hex::encode(&encoded));
    assert_eq!(encoded.to_vec(), alloy_value.to_vec());

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
    println!("Encoded Vec: {:?}", hex::encode(&encoded));
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
    let decoded_header = WasmABI::<Vec<u32>>::partial_decode(&&encoded[..], 4).unwrap();
    assert_eq!(decoded_header, (12, 20));
    assert_eq!(encoded.chunk()[12..20], vec![1, 0, 0, 0, 2, 0, 0, 0]);
}

#[test]
fn test_simple_map_sol() {
    let mut original = HashMap::new();
    original.insert(10, 20);
    original.insert(1, 5);
    original.insert(100, 60);

    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();
    println!("Encoded Map: {:?}", hex::encode(&encoded));

    let expected_encoded = hex!(
        "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000003c"
    );

    assert_eq!(encoded.to_vec(), expected_encoded);

    print_bytes::<BE, 32>(&encoded.chunk());

    let decoded = SolidityABI::<HashMap<u32, u32>>::decode(&&encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}
#[test]
fn test_simple_map_sol_abi_tmp() {
    let mut original = HashMap::new();
    original.insert(5, 6);

    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();
    println!("Encoded Map: {:?}", hex::encode(&encoded));

    // let expected_encoded = hex!(
    //     "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000003c"
    // );
    // 0   - 0000000000000000000000000000000000000000000000000000000000000020 - Смещение до начала
    // данных (32) 32  - 0000000000000000000000000000000000000000000000000000000000000003 -
    // Длина HashMap (3 элемента)
    // 64  - 0000000000000000000000000000000000000000000000000000000000000040 - Смещение до начала
    // ключей (64) 96  - 0000000000000000000000000000000000000000000000000000000000000100 -
    // Смещение до начала значений (256)

    // Ключи:
    // 128 - 0000000000000000000000000000000000000000000000000000000000000003 - Длина массива ключей
    // (3) 160 - 0000000000000000000000000000000000000000000000000000000000000001 - Ключ 1 (1)
    // 192 - 000000000000000000000000000000000000000000000000000000000000000a - Ключ 2 (10)
    // 224 - 0000000000000000000000000000000000000000000000000000000000000064 - Ключ 3 (100)

    // Значения:
    // 256 - 0000000000000000000000000000000000000000000000000000000000000003 - Длина массива
    // значений (3) 288 - 0000000000000000000000000000000000000000000000000000000000000005 -
    // Значение 1 (5) 320 - 0000000000000000000000000000000000000000000000000000000000000014 -
    // Значение 2 (20) 352 - 000000000000000000000000000000000000000000000000000000000000003c -
    // Значение 3 (60)

    // assert_eq!(encoded.to_vec(), expected_encoded);

    // println!("Encoded Map: {:?}", hex::encode(&encoded));

    // let decoded = SolidityABI::<HashMap<u32, u32>>::decode(&&encoded[..], 0).unwrap();

    // assert_eq!(decoded, original);
}

// Outer HashMap header
// 0   - 00000000000000000000000000000000000000000000000000000000000000020 // Offset to data (32)
// 32  - 00000000000000000000000000000000000000000000000000000000000000002 // Number of elements (2)
// 64  - 0000000000000000000000000000000000000000000000000000000000000040 // Offset to keys (64)
// 96  - 00000000000000000000000000000000000000000000000000000000000000e0 // Offset to values (224)

// // Outer HashMap keys
// 128 - 0000000000000000000000000000000000000000000000000000000000000002 // Number of keys (2)
// 160 - 0000000000000000000000000000000000000000000000000000000000000001 // Key 1 (1)
// 192 - 0000000000000000000000000000000000000000000000000000000000000002 // Key 2 (2)

// // Outer HashMap values (offsets to inner HashMaps)
// 224 - 0000000000000000000000000000000000000000000000000000000000000002 // Number of values (2)
// 256 - 0000000000000000000000000000000000000000000000000000000000000100 // Offset to first inner
// HashMap (256) 288 - 0000000000000000000000000000000000000000000000000000000000000180 // Offset to
// second inner HashMap (384)

// // First inner HashMap (for key 1)
// 320 - 0000000000000000000000000000000000000000000000000000000000000020 // Offset to data (32)
// 352 - 0000000000000000000000000000000000000000000000000000000000000001 // Number of elements (1)
// 384 - 0000000000000000000000000000000000000000000000000000000000000040 // Offset to keys (64)
// 416 - 0000000000000000000000000000000000000000000000000000000000000080 // Offset to values (128)
// 448 - 0000000000000000000000000000000000000000000000000000000000000001 // Number of keys (1)
// 480 - 0000000000000000000000000000000000000000000000000000000000000005 // Key (5)
// 512 - 0000000000000000000000000000000000000000000000000000000000000001 // Number of values (1)
// 544 - 0000000000000000000000000000000000000000000000000000000000000006 // Value (6)

// // Second inner HashMap (for key 2)
// 576 - 0000000000000000000000000000000000000000000000000000000000000020 // Offset to data (32)
// 608 - 0000000000000000000000000000000000000000000000000000000000000001 // Number of elements (1)
// 640 - 0000000000000000000000000000000000000000000000000000000000000040 // Offset to keys (64)
// 672 - 0000000000000000000000000000000000000000000000000000000000000080 // Offset to values (128)
// 704 - 0000000000000000000000000000000000000000000000000000000000000001 // Number of keys (1)
// 736 - 0000000000000000000000000000000000000000000000000000000000000007 // Key (7)
// 768 - 0000000000000000000000000000000000000000000000000000000000000001 // Number of values (1)
// 800 - 0000000000000000000000000000000000000000000000000000000000000008 // Value (8)
#[test]
fn test_nested_map_sol_abi1() {
    let mut original = HashMap::new();
    original.insert(9, HashMap::from([(8, 7)]));
    // original.insert(2, HashMap::from([(7, 8)]));

    println!("Original Map: {:?}", original);

    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();
    println!("Encoded Map: {:?}", hex::encode(&encoded));

    print_bytes::<BE, 32>(&encoded.chunk());

    // let expected_encoded =
    // "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000008"
    // ;

    let decoded = SolidityABI::<HashMap<u32, HashMap<u32, u32>>>::decode(&&encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}
#[test]
fn test_nested_map_sol_abi2() {
    let mut original = HashMap::new();
    original.insert(1, HashMap::from([(5, 6)]));
    original.insert(2, HashMap::from([(7, 8)]));

    let mut buf = BytesMut::new();
    SolidityABI::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    print_bytes::<BE, 32>(&encoded.chunk());

    // println!("Encoded Map: {:?}", hex::encode(&encoded));

    // let expected_encoded =
    // "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000008"
    // ;

    let decoded = SolidityABI::<HashMap<u32, HashMap<u32, u32>>>::decode(&&encoded[..], 0).unwrap();
    println!("Decoded Map: {:?}", decoded);

    assert_eq!(decoded, original);
}

#[test]
fn test_simple_map_wasm_abi() {
    let mut original = HashMap::new();
    original.insert(100, 20);
    original.insert(3, 5);
    original.insert(1000, 60);

    let mut buf = BytesMut::new();
    WasmABI::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();

    let expected_encoded = hex!(
        "03000000140000000c000000200000000c0000000300000064000000e803000005000000140000003c000000"
    );

    assert_eq!(encoded.to_vec(), expected_encoded);

    println!("Encoded Map: {:?}", hex::encode(&encoded));

    let decoded = WasmABI::<HashMap<u32, u32>>::decode(&&encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}

#[test]
fn test_nested_map_wasm_abi() {
    let mut original = HashMap::new();
    original.insert(1, HashMap::from([(5, 6)]));
    original.insert(2, HashMap::from([(7, 8)]));

    let mut buf = BytesMut::new();
    WasmABI::encode(&original, &mut buf, 0).unwrap();

    let encoded = buf.freeze();
    let expected_encoded =
    "0200000014000000080000001c0000003800000001000000020000000100000028000000040000002c00000004000000010000003000000004000000340000000400000005000000060000000700000008000000";

    assert_eq!(hex::encode(&encoded), expected_encoded, "Encoding mismatch");

    print_bytes::<LE, 4>(&encoded.chunk());

    let decoded = WasmABI::<HashMap<u32, HashMap<u32, u32>>>::decode(&&encoded[..], 0).unwrap();

    assert_eq!(decoded, original);
}
