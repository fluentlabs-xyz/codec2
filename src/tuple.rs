use crate::encoder::{align_up, ByteOrderExt, CodecError, Encoder};

use bytes::{buf, Buf, BytesMut};

impl<T: Encoder> Encoder for (T,) {
    const HEADER_SIZE: usize = 0;
    const DATA_SIZE: usize = T::DATA_SIZE;

    fn encode<B: ByteOrderExt, const ALIGN: usize>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        self.0.encode::<B, ALIGN>(buf, offset)
    }

    fn decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError> {
        Ok((T::decode::<B, ALIGN>(buf, offset)?,))
    }

    fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        T::partial_decode::<B, ALIGN>(buf, offset)
    }
}

macro_rules! impl_encoder_for_tuple {
    ($($T:ident),+; $($idx:tt),+) => {
        impl<$($T: Encoder,)+> Encoder for ($($T,)+) {
            const HEADER_SIZE: usize = $($T::HEADER_SIZE +)+ 0;
            const DATA_SIZE: usize = $($T::DATA_SIZE +)+ 0;

            fn encode<B: ByteOrderExt, const ALIGN: usize>(
                &self,
                buf: &mut BytesMut,
                offset: usize,
            ) -> Result<(), CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);
                let mut current_offset = aligned_offset;
                $(
                    self.$idx.encode::<B, ALIGN>(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + $T::HEADER_SIZE+$T::DATA_SIZE);
                )+
                Ok(())
            }

            fn decode<B: ByteOrderExt, const ALIGN: usize>(
                buf: &impl Buf,
                offset: usize,
            ) -> Result<Self, CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);
                let mut current_offset = aligned_offset;
                Ok(($(
                    {
                        let value = $T::decode::<B, ALIGN>(buf, current_offset)?;
                        current_offset = align_up::<ALIGN>(current_offset +$T::HEADER_SIZE+ $T::DATA_SIZE);
                        value
                    },
                )+))
            }

            fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
                buf: &impl Buf,
                offset: usize,
            ) -> Result<(usize, usize), CodecError> {
                let aligned_offset = align_up::<ALIGN>(offset);
                let mut total_size = 0;
                let mut current_offset = aligned_offset;
                $(
                    let (_, size) = $T::partial_decode::<B, ALIGN>(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + size);
                    total_size += size;
                )+
                Ok((offset, total_size))
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
    use byteorder::LittleEndian;

    #[test]
    fn test_single_element_tuple() {
        let original: (u32,) = (100u32,);
        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        let encoded = buffer.freeze();
        assert_eq!(hex::encode(&encoded), "64000000");

        let decoded = <(u32,)>::decode::<LittleEndian, 4>(&mut encoded.clone(), 0).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_simple_tuple() {
        type Tuple = (u32, u16);
        let original: Tuple = (100u32, 20u16);
        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        let encoded = buffer.freeze();
        println!("{:?}", encoded);
        assert_eq!(hex::encode(&encoded), "6400000014000000");

        let decoded = Tuple::decode::<LittleEndian, 4>(&mut encoded.clone(), 0).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_big_tuple() {
        type Tuple = (u32, u16, u8, u64, u32, u16, u8, u64);
        let original: Tuple = (100u32, 20u16, 30u8, 40u64, 50u32, 60u16, 70u8, 80u64);
        let mut buffer = BytesMut::new();
        original.encode::<LittleEndian, 4>(&mut buffer, 0).unwrap();

        let encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));
        assert_eq!(
            hex::encode(&encoded),
            "64000000140000001e0000002800000000000000320000003c000000460000005000000000000000"
        );

        let decoded = Tuple::decode::<LittleEndian, 4>(&mut encoded.clone(), 0).unwrap();
        assert_eq!(decoded, original);
    }
}
