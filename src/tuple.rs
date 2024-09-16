use crate::encoder::Encoder;
use byteorder::ByteOrder;
use bytes::{Bytes, BytesMut};

impl<T: Encoder> Encoder for (T,) {
    const HEADER_SIZE: usize = T::HEADER_SIZE;

    fn encode<B: ByteOrder, const ALIGN: usize>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = Self::align(offset, ALIGN);
        self.0.encode::<B, ALIGN>(buffer, aligned_offset);
    }

    fn decode_header<B: ByteOrder, const ALIGN: usize>(
        bytes: &Bytes,
        offset: usize,
    ) -> (Self, usize, usize) {
        let aligned_offset = Self::align(offset, ALIGN);
        let (value, _, size) = T::decode_header::<B, ALIGN>(bytes, aligned_offset);
        ((value,), aligned_offset, size)
    }
}

macro_rules! impl_encoder_for_tuple {
    ($($T:ident),+; $($idx:tt),+) => {
        impl<$($T: Encoder,)+> Encoder for ($($T,)+) {
            const HEADER_SIZE: usize = $($T::HEADER_SIZE +)+ 0;

            fn encode<B: ByteOrder, const ALIGN: usize>(&self, buffer: &mut BytesMut, offset: usize) {
                let aligned_offset = Self::align(offset, ALIGN);
                let mut current_offset = aligned_offset;
                $(
                    self.$idx.encode::<B, ALIGN>(buffer, current_offset);
                    current_offset += $T::HEADER_SIZE;
                )+
            }

            fn decode_header<B: ByteOrder, const ALIGN: usize>(
                bytes: &Bytes,
                offset: usize,
            ) -> (Self, usize, usize) {
                let aligned_offset = Self::align(offset, ALIGN);
                let mut current_offset = aligned_offset;
                let mut total_size = 0;
                (
                    ($(
                        {
                            let (value, _, size) = $T::decode_header::<B, ALIGN>(bytes, current_offset);
                            current_offset += size;
                            total_size += size;
                            value
                        },
                    )+),
                    aligned_offset,
                    total_size
                )
            }

            fn decode_body<B: ByteOrder, const ALIGN: usize>(bytes: &Bytes, offset: usize) -> Self {
                let (value, _, _) = Self::decode_header::<B, ALIGN>(bytes, offset);
                value
            }
        }
    };
}

impl_encoder_for_tuple!(T1, T2; 0, 1);
impl_encoder_for_tuple!(T1, T2, T3; 0, 1, 2);
impl_encoder_for_tuple!(T1, T2, T3, T4; 0, 1, 2, 3);
impl_encoder_for_tuple!(T1, T2, T3, T4, T5; 0, 1, 2, 3, 4);
impl_encoder_for_tuple!(T1, T2, T3, T4, T5, T6; 0, 1, 2, 3, 4, 5);
impl_encoder_for_tuple!(T1, T2, T3, T4, T5, T6, T7; 0, 1, 2, 3, 4, 5, 6);
impl_encoder_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8; 0, 1, 2, 3, 4, 5, 6, 7);

#[cfg(test)]
mod tests {
    use crate::encoder::Encoder;
    use byteorder::LittleEndian;
    use bytes::BytesMut;

    #[test]
    fn test_single_element_tuple() {
        let original: (u32,) = (100u32,);
        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0);

        let encoded = buffer.freeze();
        assert_eq!(hex::encode(&encoded), "64000000");

        let (decoded, _, _) = <(u32,)>::decode_header::<LittleEndian, 4>(&encoded, 0);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_simple_tuple() {
        type Tuple = (u32, u32);
        let original: Tuple = (100u32, 20u32);
        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0);

        println!("{}", hex::encode(&buffer));

        let encoded_buffer = buffer.freeze();
        let decoded = Tuple::decode_body::<LittleEndian, 4>(&encoded_buffer, 0);

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_big_tuple() {
        type Tuple = (u32, u32, u32, u32, u32, u32, u32, u32);
        let original: Tuple = (100u32, 20u32, 30u32, 40u32, 50u32, 60u32, 70u32, 80u32);
        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0);

        println!("{}", hex::encode(&buffer));

        let encoded_buffer = buffer.freeze();
        let decoded = Tuple::decode_body::<LittleEndian, 4>(&encoded_buffer, 0);

        assert_eq!(decoded, original);
    }
}
