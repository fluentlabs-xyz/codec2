use alloy_primitives::Bytes;
use bytes::{Buf, BytesMut};
use core::marker::PhantomData;

pub trait Alignment {
    const SIZE: usize;
    fn align(offset: usize) -> usize;
}

pub trait Endianness {
    fn write_u32(buffer: &mut [u8], value: u32);
    fn read_u32(buffer: &[u8]) -> u32;
    fn is_little_endian() -> bool {
        true
    }
}

pub trait Encoder<T: Sized> {
    const HEADER_SIZE: usize;

    fn encode<A: Alignment, E: Endianness>(&self, buf: &mut BytesMut, offset: usize);

    fn decode_header<A: Alignment, E: Endianness>(
        buf: &bytes::Bytes,
        offset: usize,
        result: &mut T,
    ) -> (usize, usize);

    fn decode_body<A: Alignment, E: Endianness>(buf: &bytes::Bytes, offset: usize, result: &mut T);
}

pub struct Align0;
pub struct Align1;

pub struct Align2;
pub struct Align4;
pub struct Align8;

impl Alignment for Align0 {
    const SIZE: usize = 0;
    fn align(offset: usize) -> usize {
        offset
    }
}

impl Alignment for Align1 {
    const SIZE: usize = 1;
    fn align(offset: usize) -> usize {
        offset
    }
}

impl Alignment for Align2 {
    const SIZE: usize = 2;
    fn align(offset: usize) -> usize {
        (offset + 1) & !1
    }
}

impl Alignment for Align4 {
    const SIZE: usize = 4;
    fn align(offset: usize) -> usize {
        (offset + 3) & !3
    }
}

impl Alignment for Align8 {
    const SIZE: usize = 8;
    fn align(offset: usize) -> usize {
        (offset + 7) & !7
    }
}

pub struct LittleEndian;
pub struct BigEndian;

impl Endianness for LittleEndian {
    // TODO: fix this
    // it's broken because it works only for u32 right now. We need to make it generic
    fn write_u32(buffer: &mut [u8], value: u32) {
        buffer[..4].copy_from_slice(&value.to_le_bytes());
    }
    fn read_u32(buffer: &[u8]) -> u32 {
        u32::from_le_bytes(buffer[..4].try_into().unwrap())
    }
}

impl Endianness for BigEndian {
    fn write_u32(buffer: &mut [u8], value: u32) {
        buffer[..4].copy_from_slice(&value.to_be_bytes());
    }
    fn read_u32(buffer: &[u8]) -> u32 {
        u32::from_be_bytes(buffer[..4].try_into().unwrap())
    }
    fn is_little_endian() -> bool {
        false
    }
}

pub struct FieldEncoder<T: Sized + Encoder<T>, const OFFSET: usize> {
    _phantom: PhantomData<T>,
}

impl<T: Sized + Encoder<T>, const OFFSET: usize> FieldEncoder<T, OFFSET> {
    pub const OFFSET: usize = OFFSET;
    pub const FIELD_SIZE: usize = T::HEADER_SIZE;

    pub fn decode_field_header<A: Alignment, E: Endianness>(
        buffer: &[u8],
        result: &mut T,
    ) -> (usize, usize) {
        Self::decode_field_header_at::<A, E>(buffer, Self::OFFSET, result)
    }

    pub fn decode_field_header_at<A: Alignment, E: Endianness>(
        buffer: &[u8],
        offset: usize,
        result: &mut T,
    ) -> (usize, usize) {
        let bytes = bytes::Bytes::copy_from_slice(buffer);
        T::decode_header::<A, E>(&bytes, offset, result)
    }

    pub fn decode_field_body<A: Alignment, E: Endianness>(buffer: &[u8], result: &mut T) {
        Self::decode_field_body_at::<A, E>(buffer, Self::OFFSET, result)
    }

    pub fn decode_field_body_at<A: Alignment, E: Endianness>(
        buffer: &[u8],
        offset: usize,
        result: &mut T,
    ) {
        let mut bytes = bytes::Bytes::copy_from_slice(buffer);
        T::decode_body::<A, E>(&mut bytes, offset, result)
    }
}

fn print_buffer(buffer: &[u8]) {
    for (i, &byte) in buffer.iter().enumerate() {
        print!("{:02X} ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
    println!();
}
