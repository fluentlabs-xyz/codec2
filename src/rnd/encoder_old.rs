use bytes::BytesMut;
use core::marker::PhantomData;

pub trait Alignment {
    const SIZE: usize;
    fn align(offset: usize) -> usize;
}

pub trait Encoder<T: Sized> {
    const HEADER_SIZE: usize;

    fn encode<A: Alignment, E: Endian>(&self, buf: &mut BytesMut, offset: usize);

    fn decode_header<A: Alignment, E: Endian>(
        buf: &bytes::Bytes,
        offset: usize,
        result: &mut T,
    ) -> (usize, usize);

    fn decode_body<A: Alignment, E: Endian>(buf: &bytes::Bytes, offset: usize, result: &mut T) {
        Self::decode_header::<A, E>(buf, offset, result);
    }
}

// TODO: d1r1 change Alignment to be a const generic parameter
pub struct Align0;
pub struct Align1;
pub struct Align2;
pub struct Align4;
pub struct Align8;
pub struct Align16;
pub struct Align32;
pub struct Align64;

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

impl Alignment for Align16 {
    const SIZE: usize = 16;
    fn align(offset: usize) -> usize {
        (offset + 15) & !15
    }
}

impl Alignment for Align32 {
    const SIZE: usize = 32;
    fn align(offset: usize) -> usize {
        (offset + 31) & !31
    }
}

impl Alignment for Align64 {
    const SIZE: usize = 64;
    fn align(offset: usize) -> usize {
        (offset + 63) & !63
    }
}

// ENDIANNESS

pub trait EndianConvert: Sized {
    type Bytes: AsRef<[u8]> + AsMut<[u8]> + Default;

    fn to_le_bytes(self) -> Self::Bytes;
    fn to_be_bytes(self) -> Self::Bytes;
    fn from_le_bytes(bytes: Self::Bytes) -> Self;
    fn from_be_bytes(bytes: Self::Bytes) -> Self;
}

impl EndianConvert for u32 {
    type Bytes = [u8; 4];

    fn to_le_bytes(self) -> Self::Bytes {
        self.to_le_bytes()
    }
    fn to_be_bytes(self) -> Self::Bytes {
        self.to_be_bytes()
    }
    fn from_le_bytes(bytes: Self::Bytes) -> Self {
        Self::from_le_bytes(bytes)
    }
    fn from_be_bytes(bytes: Self::Bytes) -> Self {
        Self::from_be_bytes(bytes)
    }
}

pub trait Endian {
    fn write<T: EndianConvert>(buffer: &mut [u8], value: T);
    fn read<T: EndianConvert>(buffer: &[u8]) -> T;
    fn is_little_endian() -> bool;
}

pub struct LittleEndian;
pub struct BigEndian;

impl Endian for LittleEndian {
    fn write<T: EndianConvert>(buffer: &mut [u8], value: T) {
        let bytes = value.to_le_bytes();
        let len = buffer.len().min(bytes.as_ref().len());
        buffer[..len].copy_from_slice(&bytes.as_ref()[..len]);
    }

    fn read<T: EndianConvert>(buffer: &[u8]) -> T {
        let mut bytes = T::Bytes::default();
        let len = buffer.len().min(bytes.as_ref().len());
        bytes.as_mut()[..len].copy_from_slice(&buffer[..len]);
        T::from_le_bytes(bytes)
    }

    fn is_little_endian() -> bool {
        true
    }
}

impl Endian for BigEndian {
    fn write<T: EndianConvert>(buffer: &mut [u8], value: T) {
        let bytes = value.to_be_bytes();
        let len = buffer.len().min(bytes.as_ref().len());
        let start = buffer.len() - len;
        buffer[start..].copy_from_slice(&bytes.as_ref()[bytes.as_ref().len() - len..]);
    }

    fn read<T: EndianConvert>(buffer: &[u8]) -> T {
        let mut bytes = T::Bytes::default();
        let len = buffer.len().min(bytes.as_ref().len());
        let start = bytes.as_ref().len() - len;
        bytes.as_mut()[start..].copy_from_slice(&buffer[buffer.len() - len..]);
        T::from_be_bytes(bytes)
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

    pub fn decode_field_header<A: Alignment, E: Endian>(
        buffer: &[u8],
        result: &mut T,
    ) -> (usize, usize) {
        Self::decode_field_header_at::<A, E>(buffer, Self::OFFSET, result)
    }

    pub fn decode_field_header_at<A: Alignment, E: Endian>(
        buffer: &[u8],
        offset: usize,
        result: &mut T,
    ) -> (usize, usize) {
        let bytes = bytes::Bytes::copy_from_slice(buffer);
        T::decode_header::<A, E>(&bytes, offset, result)
    }

    pub fn decode_field_body<A: Alignment, E: Endian>(buffer: &[u8], result: &mut T) {
        Self::decode_field_body_at::<A, E>(buffer, Self::OFFSET, result)
    }

    pub fn decode_field_body_at<A: Alignment, E: Endian>(
        buffer: &[u8],
        offset: usize,
        result: &mut T,
    ) {
        let bytes = bytes::Bytes::copy_from_slice(buffer);
        T::decode_body::<A, E>(&bytes, offset, result)
    }
}
