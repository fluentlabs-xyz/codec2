use crate::encoder::{Alignment, Encoder, Endianness};
use bytes::{Bytes, BytesMut};

impl<T: Encoder<T>> Encoder<(T,)> for (T,) {
    const HEADER_SIZE: usize = T::HEADER_SIZE;

    fn encode<AL: Alignment, EN: Endianness>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = AL::align(offset);
        self.0.encode::<AL, EN>(buffer, aligned_offset);
    }

    fn decode_header<AL: Alignment, EN: Endianness>(
        bytes: &Bytes,
        offset: usize,
        result: &mut (T,),
    ) -> (usize, usize) {
        let aligned_offset = AL::align(offset);
        T::decode_header::<AL, EN>(bytes, aligned_offset, &mut result.0)
    }

    fn decode_body<AL: Alignment, EN: Endianness>(bytes: &Bytes, offset: usize, result: &mut (T,)) {
        let aligned_offset = AL::align(offset);
        T::decode_body::<AL, EN>(bytes, aligned_offset, &mut result.0);
    }
}

macro_rules! impl_encoder_for_tuple {
    ($($T:ident),+; $($idx:tt),+) => {
        impl<$($T: Encoder<$T>,)+> Encoder<($($T,)+)> for ($($T,)+) {
            const HEADER_SIZE: usize = $($T::HEADER_SIZE +)+ 0;

            fn encode<AL: Alignment, EN: Endianness>(&self, buffer: &mut BytesMut, offset: usize) {
                let aligned_offset = AL::align(offset);
                let mut current_offset = aligned_offset;
                $(
                    self.$idx.encode::<AL, EN>(buffer, current_offset);
                    current_offset += $T::HEADER_SIZE;
                )+
            }

            fn decode_header<AL: Alignment, EN: Endianness>(
                bytes: &Bytes,
                offset: usize,
                result: &mut ($($T,)+),
            ) -> (usize, usize) {
                let aligned_offset = AL::align(offset);
                let mut current_offset = aligned_offset;
                let mut total_length = 0;
                $(
                    let (_, length) = $T::decode_header::<AL, EN>(bytes, current_offset, &mut result.$idx);
                    current_offset += $T::HEADER_SIZE;
                    total_length += length;
                )+
                (aligned_offset, total_length)
            }

            fn decode_body<AL: Alignment, EN: Endianness>(
                bytes: &Bytes,
                offset: usize,
                result: &mut ($($T,)+),
            ) {
                let aligned_offset = AL::align(offset);
                let mut current_offset = aligned_offset;
                $(
                    $T::decode_body::<AL, EN>(bytes, current_offset, &mut result.$idx);
                    current_offset += $T::HEADER_SIZE;
                )+
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
    use super::*;
    use crate::encoder::{Align4, LittleEndian};

    #[test]
    fn test_single_element_tuple() {
        type SingleTuple = (u32,);
        let mut buffer = BytesMut::new();
        (100u32,).encode::<Align4, LittleEndian>(&mut buffer, 0);
        println!("{}", hex::encode(&buffer));
        let encoded_buffer = buffer.freeze();
        let mut result: SingleTuple = Default::default();
        SingleTuple::decode_body::<Align4, LittleEndian>(&encoded_buffer, 0, &mut result);
        assert_eq!(result, (100,));
    }

    #[test]
    fn test_simple_tuple() {
        type Tuple = (u32, u32);
        let mut buffer = BytesMut::new();
        (100u32, 20u32).encode::<Align4, LittleEndian>(&mut buffer, 0);
        println!("{}", hex::encode(&buffer));
        let encoded_buffer = buffer.freeze();
        let mut result: Tuple = Default::default();
        Tuple::decode_body::<Align4, LittleEndian>(&encoded_buffer, 0, &mut result);
        assert_eq!(result, (100, 20));
    }

    #[test]
    fn test_big_tuple() {
        type Tuple = (u32, u32, u32, u32, u32, u32, u32, u32);
        let mut buffer = BytesMut::new();
        (100u32, 20u32, 30u32, 40u32, 50u32, 60u32, 70u32, 80u32)
            .encode::<Align4, LittleEndian>(&mut buffer, 0);
        println!("{}", hex::encode(&buffer));

        let encoded_buffer = buffer.freeze();
        let mut result: Tuple = Default::default();
        Tuple::decode_body::<Align4, LittleEndian>(&encoded_buffer, 0, &mut result);
        assert_eq!(result, (100, 20, 30, 40, 50, 60, 70, 80));
    }
}
