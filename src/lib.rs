use bytes::BytesMut;
use core::marker::PhantomData;

// Маркерные типы для endianness
pub struct LittleEndian;
pub struct BigEndian;

// Трейт для типов, поддерживающих кодирование
pub trait Encoder: Sized {
    // Методы для кодирования в конкретном endianness
    fn encode_le(&self, buf: &mut [u8]);
    fn encode_be(&self, buf: &mut [u8]);

    // Обобщенный метод encode с выбором endianness через generic параметр
    fn encode<E: Endianness>(&self, buf: &mut [u8]) {
        E::encode(self, buf);
    }

    // Метод для определения размера закодированного значения
    fn encoded_size(&self) -> usize;
}

// Трейт для определения endianness
pub trait Endianness {
    fn encode<T: Encoder>(value: &T, buf: &mut [u8]);
}

impl Endianness for LittleEndian {
    fn encode<T: Encoder>(value: &T, buf: &mut [u8]) {
        value.encode_le(buf);
    }
}

impl Endianness for BigEndian {
    fn encode<T: Encoder>(value: &T, buf: &mut [u8]) {
        value.encode_be(buf);
    }
}

// Пример реализации для u32
impl Encoder for u32 {
    fn encode_le(&self, buf: &mut [u8]) {
        buf[..4].copy_from_slice(&self.to_le_bytes());
    }

    fn encode_be(&self, buf: &mut [u8]) {
        buf[..4].copy_from_slice(&self.to_be_bytes());
    }

    fn encoded_size(&self) -> usize {
        4
    }
}

// Структура кодека
pub struct MyCodec<T: Encoder> {
    value: T,
}

impl<T: Encoder> MyCodec<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn encode<E: Endianness>(&self, buf: &mut BytesMut) {
        let size = self.value.encoded_size();
        if buf.len() < size {
            buf.resize(size, 0);
        }
        self.value.encode::<E>(&mut buf[..size]);
    }
}

// Макрос для упрощения реализации Encoder для примитивных типов
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

// Реализация для других примитивных типов
impl_encoder_for_primitive!(u16);
impl_encoder_for_primitive!(u64);
impl_encoder_for_primitive!(i32);
impl_encoder_for_primitive!(i64);

// Пример использования
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec() {
        let value: u32 = 0x12345678;
        let codec = MyCodec::new(value);
        let mut buf = BytesMut::with_capacity(4);

        codec.encode::<LittleEndian>(&mut buf);
        assert_eq!(buf.as_ref(), &[0x78, 0x56, 0x34, 0x12]);

        buf.clear();
        codec.encode::<BigEndian>(&mut buf);
        assert_eq!(buf.as_ref(), &[0x12, 0x34, 0x56, 0x78]);
    }
}
