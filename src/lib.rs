#![no_std]

use bytes::BytesMut;

// Маркерные типы для endianness
pub struct LittleEndian;
pub struct BigEndian;

pub trait Endianness {
    fn write_bytes<T: Encoder + ?Sized>(value: &T, buf: &mut [u8]);
}

impl Endianness for LittleEndian {
    fn write_bytes<T: Encoder + ?Sized>(value: &T, buf: &mut [u8]) {
        value.encode_le(buf);
    }
}

impl Endianness for BigEndian {
    fn write_bytes<T: Encoder + ?Sized>(value: &T, buf: &mut [u8]) {
        value.encode_be(buf);
    }
}

pub trait Encoder: Sized {
    fn encode_le(&self, buf: &mut [u8]);
    fn encode_be(&self, buf: &mut [u8]);
    fn encoded_size(&self) -> usize;

    fn encode<E: Endianness, const ALIGNMENT: usize>(&self, buf: &mut BytesMut, offset: usize) {
        let size = self.encoded_size();
        let aligned_size = (size + ALIGNMENT - 1) / ALIGNMENT * ALIGNMENT;

        if buf.len() < offset + aligned_size {
            buf.resize(offset + aligned_size, 0);
        }

        // Заполняем буфер нулями для выравнивания
        buf[offset..offset + aligned_size].fill(0);

        // Кодируем значение
        E::write_bytes(self, &mut buf[offset..offset + size]);
    }
}

// Макрос для реализации Encoder для примитивных типов
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

            fn encoded_size(&self) -> usize {
                core::mem::size_of::<$type>()
            }
        }
    };
}

// Реализация для примитивных типов
impl_encoder_for_primitive!(u8);
impl_encoder_for_primitive!(u16);
impl_encoder_for_primitive!(u32);
impl_encoder_for_primitive!(u64);
impl_encoder_for_primitive!(i8);
impl_encoder_for_primitive!(i16);
impl_encoder_for_primitive!(i32);
impl_encoder_for_primitive!(i64);

// Пример макроса для реализации Encoder для пользовательских структур
#[macro_export]
macro_rules! impl_encoder_for_struct {
    ($struct:ident, $($field:ident),+) => {
        impl Encoder for $struct {
            fn encode_le(&self, buf: &mut [u8]) {
                let mut offset = 0;
                $(
                    self.$field.encode_le(&mut buf[offset..]);
                    offset += self.$field.encoded_size();
                )+
            }

            fn encode_be(&self, buf: &mut [u8]) {
                let mut offset = 0;
                $(
                    self.$field.encode_be(&mut buf[offset..]);
                    offset += self.$field.encoded_size();
                )+
            }

            fn encoded_size(&self) -> usize {
                0 $(+ self.$field.encoded_size())+
            }
        }
    };
}

// Пример использования
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_encoding() {
        let value: u16 = 0x1234;
        let mut buf = BytesMut::with_capacity(4);

        value.encode::<LittleEndian, 4>(&mut buf, 0); // Выравнивание до 4 байт
        assert_eq!(buf.as_ref(), &[0x34, 0x12, 0x00, 0x00]);

        buf.clear();
        value.encode::<BigEndian, 4>(&mut buf, 0);
        assert_eq!(buf.as_ref(), &[0x12, 0x34, 0x00, 0x00]);
    }

    #[test]
    fn test_struct_encoding() {
        struct TestStruct {
            a: u16,
            b: u32,
        }

        impl_encoder_for_struct!(TestStruct, a, b);

        let value = TestStruct {
            a: 0x1234,
            b: 0x56789ABC,
        };
        let mut buf = BytesMut::with_capacity(8);

        value.encode::<LittleEndian, 8>(&mut buf, 0); // Выравнивание до 8 байт
        assert_eq!(
            buf.as_ref(),
            &[0x34, 0x12, 0xBC, 0x9A, 0x78, 0x56, 0x00, 0x00]
        );
    }
}
