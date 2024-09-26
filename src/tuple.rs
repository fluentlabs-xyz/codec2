use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

use crate::encoder::{align_up, get_aligned_slice, Encoder};
use crate::error::CodecError;

impl<T, B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool> Encoder<B, { ALIGN }, { SOL_MODE }>
    for (T,)
where
    T: Encoder<B, { ALIGN }, { SOL_MODE }>,
{
    const HEADER_SIZE: usize = T::HEADER_SIZE;

    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        self.0.encode(buf, offset)
    }

    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        Ok((T::decode(buf, offset)?,))
    }

    fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        T::partial_decode(buf, offset)
    }
}

macro_rules! impl_encoder_for_tuple {
    ($($T:ident),+; $($idx:tt),+) => {
        impl<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool, $($T,)+> Encoder<B, {ALIGN}, {SOL_MODE}> for ($($T,)+)
        where
            $($T: Encoder<B, {ALIGN}, {SOL_MODE}>,)+
        {
            const HEADER_SIZE: usize = $($T::HEADER_SIZE +)+ 0;

            fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
                let mut current_offset = align_up::<ALIGN>(offset);
                $(
                    self.$idx.encode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + $T::HEADER_SIZE);
                )+
                Ok(())
            }

            fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
                let mut current_offset = align_up::<ALIGN>(offset);
                Ok(($(
                    {
                        let value = $T::decode(buf, current_offset)?;
                        current_offset = align_up::<ALIGN>(current_offset + $T::HEADER_SIZE);
                        value
                    },
                )+))
            }

            fn partial_decode(buf: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
                let mut total_size = 0;
                let mut current_offset = align_up::<ALIGN>(offset);
                $(
                    let (_, size) = $T::partial_decode(buf, current_offset)?;
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
    use byteorder::LittleEndian;
    use bytes::BytesMut;

    use super::*;

    #[test]
    fn test_single_element_tuple() {
        let original: (u32,) = (100u32,);
        let mut buf = BytesMut::new();
        <(u32,) as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        assert_eq!(hex::encode(&encoded), "64000000");

        let decoded = <(u32,) as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_simple_tuple() {
        type Tuple = (u32, u16);
        let original: Tuple = (100u32, 20u16);
        let mut buf = BytesMut::new();
        <Tuple as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("{:?}", encoded);
        assert_eq!(hex::encode(&encoded), "6400000014000000");

        let decoded = <Tuple as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_big_tuple() {
        type Tuple = (u32, u16, u8, u64, u32, u16, u8, u64);
        let original: Tuple = (100u32, 20u16, 30u8, 40u64, 50u32, 60u16, 70u8, 80u64);
        let mut buf = BytesMut::new();
        <Tuple as Encoder<LittleEndian, 4, false>>::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("{:?}", hex::encode(&encoded));
        assert_eq!(
            hex::encode(&encoded),
            "64000000140000001e0000002800000000000000320000003c000000460000005000000000000000"
        );

        let decoded = <Tuple as Encoder<LittleEndian, 4, false>>::decode(&encoded, 0).unwrap();
        assert_eq!(decoded, original);
    }
}
