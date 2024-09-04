// #![no_std]

use bytes::BytesMut;

pub struct LittleEndian;
pub struct BigEndian;

// Alignment types with size parameters
pub struct AlignTrailing<const N: usize>;
pub struct AlignLeading<const N: usize>;

// Trait for alignment types
pub trait Alignment {
    fn align<T: Encoder>(value: &T, buf: &mut [u8]);
    const SIZE: usize;
}

impl<const N: usize> Alignment for AlignTrailing<N> {
    fn align<T: Encoder>(value: &T, buf: &mut [u8]) {
        let size = value.encoded_size();
        assert!(
            N >= size,
            "Alignment size must be greater than or equal to the value size"
        );
        // Write value at the beginning
        value.encode_le(&mut buf[..size]);
        // Fill remaining bytes with zeros
        buf[size..N].fill(0);
    }
    const SIZE: usize = N;
}

impl<const N: usize> Alignment for AlignLeading<N> {
    fn align<T: Encoder>(value: &T, buf: &mut [u8]) {
        let size = value.encoded_size();
        assert!(
            N >= size,
            "Alignment size must be greater than or equal to the value size"
        );
        // Fill leading bytes with zeros
        buf[..N - size].fill(0);
        // Write value at the end
        value.encode_le(&mut buf[N - size..N]);
    }
    const SIZE: usize = N;
}
pub trait Endianness {
    fn write_bytes<T: Encoder + ?Sized>(value: &T, buf: &mut [u8]);
    fn read_bytes<T: Encoder + ?Sized>(buf: &[u8]) -> T;
}

impl Endianness for LittleEndian {
    fn write_bytes<T: Encoder + ?Sized>(value: &T, buf: &mut [u8]) {
        value.encode_le(buf);
    }
    fn read_bytes<T: Encoder + ?Sized>(buf: &[u8]) -> T {
        T::decode_le(buf)
    }
}

impl Endianness for BigEndian {
    fn write_bytes<T: Encoder + ?Sized>(value: &T, buf: &mut [u8]) {
        value.encode_be(buf);
    }
    fn read_bytes<T: Encoder + ?Sized>(buf: &[u8]) -> T {
        T::decode_be(buf)
    }
}

pub trait Encoder: Sized {
    fn encode_le(&self, buf: &mut [u8]);
    fn encode_be(&self, buf: &mut [u8]);
    fn decode_le(buf: &[u8]) -> Self;
    fn decode_be(buf: &[u8]) -> Self;
    fn encoded_size(&self) -> usize;

    fn encode_aligned<E: Endianness, A: Alignment>(&self, buf: &mut BytesMut, offset: usize) {
        let aligned_size = A::SIZE;
        if buf.len() < offset + aligned_size {
            buf.resize(offset + aligned_size, 0);
        }
        A::align(self, &mut buf[offset..offset + aligned_size]);
    }

    fn decode_aligned<E: Endianness, A: Alignment>(buf: &[u8], offset: usize) -> Self {
        let aligned_size = A::SIZE;
        assert!(
            buf.len() >= offset + aligned_size,
            "Buffer too small for aligned decoding"
        );
        let size = core::mem::size_of::<Self>();

        if aligned_size == size {
            // Если выравнивание совпадает с размером, читаем напрямую
            E::read_bytes(&buf[offset..offset + size])
        } else if aligned_size > size {
            if offset == 0 {
                // Для AlignTrailing: читаем сначала буфера
                E::read_bytes(&buf[offset..offset + size])
            } else {
                // Для AlignLeading: читаем с конца выровненного блока
                E::read_bytes(&buf[offset + aligned_size - size..offset + aligned_size])
            }
        } else {
            panic!("Invalid alignment size");
        }
    }
}

#[macro_export]
macro_rules! impl_encoder_for_primitive {
    ($type:ty) => {
        impl Encoder for $type {
            fn encode_le(&self, buf: &mut [u8]) {
                buf[..core::mem::size_of::<$type>()].copy_from_slice(&self.to_le_bytes());
            }
            fn encode_be(&self, buf: &mut [u8]) {
                buf[..core::mem::size_of::<$type>()].copy_from_slice(&self.to_be_bytes());
            }
            fn decode_le(buf: &[u8]) -> Self {
                <$type>::from_le_bytes(buf[..core::mem::size_of::<$type>()].try_into().unwrap())
            }
            fn decode_be(buf: &[u8]) -> Self {
                <$type>::from_be_bytes(buf[..core::mem::size_of::<$type>()].try_into().unwrap())
            }
            fn encoded_size(&self) -> usize {
                core::mem::size_of::<$type>()
            }
        }
    };
}

impl_encoder_for_primitive!(u8);
impl_encoder_for_primitive!(u16);
impl_encoder_for_primitive!(u32);
impl_encoder_for_primitive!(u64);
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixed_alignment() {
        let mut buf = BytesMut::with_capacity(16);

        // Encode with different alignments
        (0x1234u16).encode_aligned::<LittleEndian, AlignTrailing<4>>(&mut buf, 0);
        (0x56789ABCu32).encode_aligned::<LittleEndian, AlignLeading<8>>(&mut buf, 4);

        assert_eq!(
            buf.as_ref(),
            &[0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xBC, 0x9A, 0x78, 0x56]
        );

        // Decode with matching alignments
        let decoded_u16 = u16::decode_aligned::<LittleEndian, AlignTrailing<4>>(&buf, 0);
        let decoded_u32 = u32::decode_aligned::<LittleEndian, AlignLeading<8>>(&buf, 4);

        assert_eq!(decoded_u32, 0x56789ABC);
        assert_eq!(decoded_u16, 0x1234);
    }
}
