use core::marker::PhantomData;

pub trait Endianness {
    fn to_bytes(value: u32) -> [u8; 4];
}

pub struct LittleEndian;
pub struct BigEndian;

impl Endianness for LittleEndian {
    fn to_bytes(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }
}

impl Endianness for BigEndian {
    fn to_bytes(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }
}

pub trait Encodable {
    fn encode<E: Endianness>(&self, buf: &mut [u8], offset: usize);
}

impl Encodable for u32 {
    fn encode<E: Endianness>(&self, buf: &mut [u8], offset: usize) {
        let bytes = E::to_bytes(*self);
        buf[offset..offset + 4].copy_from_slice(&bytes);
    }
}

pub struct MyCodec<T: Encodable> {
    value: T,
}

impl<T: Encodable> MyCodec<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn encode<E: Endianness>(&self, buf: &mut [u8], offset: usize) {
        self.value.encode::<E>(buf, offset);
    }
}

// Пример использования
fn main() {
    let value: u32 = 0x12345678;
    let codec = MyCodec::new(value);
    let mut buf = [0u8; 4];

    codec.encode::<LittleEndian>(&mut buf, 0);
    assert_eq!(buf, [0x78, 0x56, 0x34, 0x12]);

    codec.encode::<BigEndian>(&mut buf, 0);
    assert_eq!(buf, [0x12, 0x34, 0x56, 0x78]);
}
