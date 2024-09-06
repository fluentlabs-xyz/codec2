use bytes::{Bytes, BytesMut};
use std::mem::size_of;

pub trait Endianness {
    fn convert<T: EndianConvert>(value: T) -> T;
}

pub struct LittleEndian;
pub struct BigEndian;

impl Endianness for LittleEndian {
    fn convert<T: EndianConvert>(value: T) -> T {
        value.to_le()
    }
}

impl Endianness for BigEndian {
    fn convert<T: EndianConvert>(value: T) -> T {
        value.to_be()
    }
}

pub trait EndianConvert: Sized {
    fn to_le(self) -> Self;
    fn to_be(self) -> Self;
    fn from_le(self) -> Self;
    fn from_be(self) -> Self;
}

pub trait ByteConvertible: Sized {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

// Implementation for primitive types
macro_rules! impl_endian_byte_convertible {
    ($($t:ty),*) => {
        $(
            impl EndianConvert for $t {
                fn to_le(self) -> Self { self.to_le() }
                fn to_be(self) -> Self { self.to_be() }
                fn from_le(self) -> Self { <$t>::from_le(self) }
                fn from_be(self) -> Self { <$t>::from_be(self) }
            }

            impl ByteConvertible for $t {
                fn to_bytes(&self) -> Vec<u8> { self.to_ne_bytes().to_vec() }
                fn from_bytes(bytes: &[u8]) -> Self { <$t>::from_ne_bytes(bytes.try_into().unwrap()) }
            }
        )*
    }
}

impl_endian_byte_convertible!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

pub trait Encoder<T: Sized>: Sized {
    const HEADER_SIZE: usize;

    fn encode<E: Endianness, A: Alignment>(&self, buffer: &mut BytesMut, offset: usize);
    fn decode_header<E: Endianness, A: Alignment>(
        bytes: &Bytes,
        offset: usize,
        result: &mut T,
    ) -> (usize, usize);
}

impl<T: EndianConvert + ByteConvertible + Copy> Encoder<T> for T {
    const HEADER_SIZE: usize = 0;

    fn encode<E: Endianness, A: Alignment>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_size = std::cmp::max(size_of::<T>(), A::SIZE);
        buffer.resize(offset + aligned_size, 0);
        let slice = &mut buffer[offset..offset + aligned_size];
        let value = E::convert(*self);
        let value_bytes = value.to_bytes();
        A::align(slice, size_of::<T>(), &value_bytes);
    }

    fn decode_header<E: Endianness, A: Alignment>(
        bytes: &Bytes,
        offset: usize,
        result: &mut T,
    ) -> (usize, usize) {
        let aligned_size = std::cmp::max(size_of::<T>(), A::SIZE);
        let slice = &bytes[offset..offset + aligned_size];
        let value_bytes = A::unalign(slice, size_of::<T>());
        *result = E::convert(T::from_bytes(&value_bytes));
        (aligned_size, 0)
    }
}

pub trait Alignment {
    const SIZE: usize;
    fn align(buf: &mut [u8], data_size: usize, data: &[u8]);
    fn unalign(buf: &[u8], data_size: usize) -> Vec<u8>;
}

pub struct AlignTrailing<const N: usize>;
pub struct AlignLeading<const N: usize>;

impl<const N: usize> Alignment for AlignTrailing<N> {
    const SIZE: usize = N;
    fn align(buf: &mut [u8], data_size: usize, data: &[u8]) {
        buf[..data_size].copy_from_slice(data);
        buf[data_size..].fill(0);
    }
    fn unalign(buf: &[u8], data_size: usize) -> Vec<u8> {
        buf[..data_size].to_vec()
    }
}

impl<const N: usize> Alignment for AlignLeading<N> {
    const SIZE: usize = N;
    fn align(buf: &mut [u8], data_size: usize, data: &[u8]) {
        buf[..N - data_size].fill(0);
        buf[N - data_size..].copy_from_slice(data);
    }
    fn unalign(buf: &[u8], data_size: usize) -> Vec<u8> {
        buf[N - data_size..].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_encoding_trailing() {
        let value: u32 = 0x12345678;
        let mut buffer = BytesMut::new();
        value.encode::<LittleEndian, AlignTrailing<8>>(&mut buffer, 0);
        assert_eq!(buffer.len(), 8);
        assert_eq!(&buffer[0..4], &[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(&buffer[4..8], &[0, 0, 0, 0]);

        let mut decoded: u32 = 0;
        let bytes = buffer.freeze();
        u32::decode_header::<LittleEndian, AlignTrailing<8>>(&bytes, 0, &mut decoded);
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_primitive_encoding_leading() {
        let value: u32 = 0x12345678;
        let mut buffer = BytesMut::new();
        value.encode::<BigEndian, AlignLeading<8>>(&mut buffer, 0);
        assert_eq!(buffer.len(), 8);
        assert_eq!(&buffer[0..4], &[0, 0, 0, 0]);
        assert_eq!(&buffer[4..8], &[0x12, 0x34, 0x56, 0x78]);

        let mut decoded: u32 = 0;
        let bytes = buffer.freeze();
        u32::decode_header::<BigEndian, AlignLeading<8>>(&bytes, 0, &mut decoded);
        assert_eq!(decoded, value);
    }
}
