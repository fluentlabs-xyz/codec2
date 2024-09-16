use byteorder::ByteOrder;
use bytes::{Buf, BufMut};

// Errors
pub enum EncodingErr {
    NotEnoughSpace,
}

// TODO rm macro field encoder/decoder

// Default here can be a replace for maybe uninit
// TODO: check if it will be more efficient to decode result as maybe uninit
pub trait Encoder: Sized + Default {
    const HEADER_SIZE: usize;

    fn encode<B: ByteOrder, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncodingErr>;

    fn decode<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, EncodingErr>;

    fn partial_decode<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncodingErr>;
}

pub const fn align<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}
