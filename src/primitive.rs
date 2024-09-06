use crate::encoder2::{Align1, Alignment, Encoder, Endianness};
use bytes::{Buf, BufMut, Bytes, BytesMut};

impl Encoder<u8> for u8 {
    const HEADER_SIZE: usize = core::mem::size_of::<u8>();

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        let aligned_offset = A::align(field_offset);
        if buffer.len() < aligned_offset + Self::HEADER_SIZE {
            buffer.resize(aligned_offset + Self::HEADER_SIZE, 0);
        }
        buffer[aligned_offset] = *self;
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut u8,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        if bytes.len() < aligned_offset + Self::HEADER_SIZE {
            return (0, 0);
        }
        *result = bytes[aligned_offset];
        (0, Self::HEADER_SIZE)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut u8,
    ) {
        Self::decode_header::<A, E>(bytes, field_offset, result);
    }
}

impl Encoder<bool> for bool {
    const HEADER_SIZE: usize = core::mem::size_of::<bool>();

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        let aligned_offset = A::align(field_offset);
        let total_size = aligned_offset + A::SIZE;
        if buffer.len() < total_size {
            buffer.resize(total_size, 0);
        }
        // Заполняем нулями от field_offset до aligned_offset
        for i in field_offset..aligned_offset {
            buffer[i] = 0;
        }
        buffer[aligned_offset] = *self as u8;
        // Заполняем оставшиеся байты выравнивания нулями
        for i in (aligned_offset + 1)..total_size {
            buffer[i] = 0;
        }
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut bool,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        if bytes.len() <= aligned_offset {
            return (0, 0);
        }
        *result = bytes[aligned_offset] != 0;
        (0, A::SIZE)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut bool,
    ) {
        Self::decode_header::<A, E>(bytes, field_offset, result);
    }
}

macro_rules! impl_int {
    ($typ:ty) => {
        impl Encoder<$typ> for $typ {
            const HEADER_SIZE: usize = core::mem::size_of::<$typ>();

            fn encode<A: Alignment, E: Endianness>(
                &self,
                buffer: &mut BytesMut,
                field_offset: usize,
            ) {
                let aligned_offset = A::align(field_offset);
                let total_size = aligned_offset + A::SIZE.max(Self::HEADER_SIZE);
                if buffer.len() < total_size {
                    buffer.resize(total_size, 0);
                }
                // Заполняем нулями от field_offset до aligned_offset
                for i in field_offset..aligned_offset {
                    buffer[i] = 0;
                }
                let bytes = if E::is_little_endian() {
                    self.to_le_bytes()
                } else {
                    self.to_be_bytes()
                };
                buffer[aligned_offset..aligned_offset + Self::HEADER_SIZE].copy_from_slice(&bytes);
                // Заполняем оставшиеся байты выравнивания нулями
                for i in (aligned_offset + Self::HEADER_SIZE)..total_size {
                    buffer[i] = 0;
                }
            }

            fn decode_header<A: Alignment, E: Endianness>(
                bytes: &Bytes,
                field_offset: usize,
                result: &mut $typ,
            ) -> (usize, usize) {
                let aligned_offset = A::align(field_offset);
                if bytes.len() < aligned_offset + Self::HEADER_SIZE {
                    return (0, 0);
                }
                let slice = &bytes[aligned_offset..aligned_offset + Self::HEADER_SIZE];
                *result = if E::is_little_endian() {
                    <$typ>::from_le_bytes(slice.try_into().unwrap())
                } else {
                    <$typ>::from_be_bytes(slice.try_into().unwrap())
                };
                (0, A::SIZE.max(Self::HEADER_SIZE))
            }

            fn decode_body<A: Alignment, E: Endianness>(
                bytes: &Bytes,
                field_offset: usize,
                result: &mut $typ,
            ) {
                Self::decode_header::<A, E>(bytes, field_offset, result);
            }
        }
    };
}

impl_int!(u16);
impl_int!(u32);
impl_int!(u64);
impl_int!(i16);
impl_int!(i32);
impl_int!(i64);

impl<T: Sized + Encoder<T> + Default> Encoder<Option<T>> for Option<T> {
    const HEADER_SIZE: usize = 1 + T::HEADER_SIZE;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        let aligned_offset = A::align(field_offset);
        let total_size = aligned_offset + A::SIZE.max(Self::HEADER_SIZE);
        if buffer.len() < total_size {
            buffer.resize(total_size, 0);
        }
        // Заполняем нулями от field_offset до aligned_offset
        for i in field_offset..aligned_offset {
            buffer[i] = 0;
        }
        let option_flag = if self.is_some() { 1u8 } else { 0u8 };
        buffer[aligned_offset] = option_flag;
        if let Some(value) = self {
            value.encode::<Align1, E>(buffer, aligned_offset + 1);
        } else {
            T::default().encode::<Align1, E>(buffer, aligned_offset + 1);
        }
        // Заполняем оставшиеся байты выравнивания нулями
        for i in (aligned_offset + Self::HEADER_SIZE)..total_size {
            buffer[i] = 0;
        }
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut Option<T>,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        if bytes.len() < aligned_offset + 1 {
            return (0, 0);
        }
        let option_flag = bytes[aligned_offset];

        if option_flag != 0 {
            let mut inner_value = T::default();
            let (_, size) =
                T::decode_header::<Align1, E>(bytes, aligned_offset + 1, &mut inner_value);
            *result = Some(inner_value);
            (0, A::SIZE.max(1 + size))
        } else {
            *result = None;
            (0, A::SIZE.max(Self::HEADER_SIZE))
        }
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut Option<T>,
    ) {
        Self::decode_header::<A, E>(bytes, field_offset, result);
    }
}

impl<T: Sized + Encoder<T>, const N: usize> Encoder<[T; N]> for [T; N] {
    const HEADER_SIZE: usize = T::HEADER_SIZE * N;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, field_offset: usize) {
        for (i, item) in self.iter().enumerate() {
            item.encode::<A, E>(buffer, field_offset + i * T::HEADER_SIZE);
        }
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut [T; N],
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        if bytes.len() < aligned_offset + Self::HEADER_SIZE {
            return (0, 0);
        }
        for (i, item) in result.iter_mut().enumerate() {
            T::decode_body::<A, E>(bytes, aligned_offset + i * T::HEADER_SIZE, item);
        }
        (0, Self::HEADER_SIZE)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut [T; N],
    ) {
        Self::decode_header::<A, E>(bytes, field_offset, result);
    }
}

#[cfg(test)]
mod tests {
    use std::i64;

    use super::*;
    use crate::encoder2::{Align1, Align2, Align4, Align8, BigEndian, LittleEndian};

    #[test]
    fn test_u8_encode_decode() {
        let original: u8 = 42;
        let mut buffer = BytesMut::new();

        original.encode::<Align1, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[42]));

        let mut decoded: u8 = 0;
        u8::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_bool_encode_decode_align8() {
        let original = true;
        let mut buffer = BytesMut::new();

        original.encode::<Align8, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[1, 0, 0, 0, 0, 0, 0, 0]));

        let mut decoded = false;
        bool::decode_body::<Align8, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bool_encode_decode_no_alignment() {
        let original = true;
        let mut buffer = BytesMut::new();

        original.encode::<Align1, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[1]));

        let mut decoded = false;
        bool::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_bool_encode_decode_with_alignment() {
        let original = true;
        let mut buffer = BytesMut::new();

        original.encode::<Align4, BigEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[1, 0, 0, 0]));

        let mut decoded = false;
        bool::decode_body::<Align4, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_encode_decode_le() {
        let original: u32 = 0x12345678;
        let mut buffer = BytesMut::new();

        original.encode::<Align4, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[0x78, 0x56, 0x34, 0x12]));

        let mut decoded: u32 = 0;
        u32::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_u32_encode_decode_be() {
        let original: u32 = 0x12345678;
        let mut buffer = BytesMut::new();

        original.encode::<Align4, BigEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[0x12, 0x34, 0x56, 0x78]));

        let mut decoded: u32 = 0;
        u32::decode_body::<Align4, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_i64_encode_decode_aligned_le() {
        let original: i64 = i64::MIN;
        let mut buffer = BytesMut::new();

        original.encode::<Align8, LittleEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[0, 0, 0, 0, 0, 0, 0, 0x80]));

        let mut decoded: i64 = 0;
        i64::decode_body::<Align8, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_i64_encode_decode_aligned_be() {
        let original: i64 = i64::MIN;
        let mut buffer = BytesMut::new();

        original.encode::<Align8, BigEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(encoded, Bytes::from_static(&[0x80, 0, 0, 0, 0, 0, 0, 0]));

        let mut decoded: i64 = 0;
        i64::decode_body::<Align8, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_option_u16_encode_decode() {
        let original: Option<u16> = Some(1234);
        let mut buffer = BytesMut::new();

        original.encode::<Align8, BigEndian>(&mut buffer, 0);

        let encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));
        assert_eq!(encoded, Bytes::from_static(&[1, 4, 210, 0, 0, 0, 0, 0])); // 1 (Some), then 1234 in big-endian

        let mut decoded: Option<u16> = None;
        Option::<u16>::decode_body::<Align8, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_i64_encode_different_alignments() {
        let original: i64 = i64::MIN; // -9223372036854775808

        // Align1
        let mut buffer1 = BytesMut::new();
        original.encode::<Align1, LittleEndian>(&mut buffer1, 0);
        let encoded1 = buffer1.freeze();
        assert_eq!(
            encoded1,
            Bytes::from_static(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])
        );

        // Align4
        let mut buffer4 = BytesMut::new();
        original.encode::<Align4, LittleEndian>(&mut buffer4, 0);
        let encoded4 = buffer4.freeze();
        assert_eq!(
            encoded4,
            Bytes::from_static(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])
        );

        // Align8
        let mut buffer8 = BytesMut::new();
        original.encode::<Align8, LittleEndian>(&mut buffer8, 0);
        let encoded8 = buffer8.freeze();
        assert_eq!(
            encoded8,
            Bytes::from_static(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])
        );

        // Align4 с смещением 1
        let mut buffer4_offset = BytesMut::new();
        original.encode::<Align4, LittleEndian>(&mut buffer4_offset, 1);
        let encoded4_offset = buffer4_offset.freeze();
        assert_eq!(
            encoded4_offset,
            Bytes::from_static(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80
            ])
        );

        // Align8 с смещением 3
        let mut buffer8_offset = BytesMut::new();
        original.encode::<Align8, LittleEndian>(&mut buffer8_offset, 3);
        let encoded8_offset = buffer8_offset.freeze();
        assert_eq!(
            encoded8_offset,
            Bytes::from_static(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x80
            ])
        );
    }
}
