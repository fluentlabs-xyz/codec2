use byteorder::ByteOrder;
use bytes::{Buf, BufMut};
use std::mem::MaybeUninit;

#[derive(Debug)]
pub enum EncoderError {
    BufferTooSmall,
    InsufficientSpaceForHeader { required: usize, available: usize },
    // Add other error variants as needed
}

pub trait Encoder<const HEADER_SIZE: usize>: Sized + Default {
    #[inline]
    fn encode<B: ByteOrder, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError> {
        let aligned_offset = Self::check_buffer_size_for_encoding::<ALIGN>(buf, offset)?;
        self.encode_inner::<B, ALIGN>(buf, aligned_offset)
    }

    fn encode_inner<B: ByteOrder, const ALIGN: usize>(
        &self,
        buf: &mut impl BufMut,
        offset: usize,
    ) -> Result<(), EncoderError>;

    #[inline]
    fn decode<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError> {
        let aligned_offset = Self::check_buffer_size_for_decoding::<ALIGN>(buf, offset)?;
        Self::decode_inner::<B, ALIGN>(buf, aligned_offset)
    }

    fn decode_inner<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError>;

    fn partial_decode<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), EncoderError>;

    #[inline]
    fn total_encoded_size<const ALIGN: usize>(&self) -> usize {
        align::<ALIGN>(HEADER_SIZE) + self.data_size()
    }

    fn data_size(&self) -> usize;

    #[inline]
    fn check_buffer_size_for_encoding<const ALIGN: usize>(
        buf: &impl BufMut,
        offset: usize,
    ) -> Result<usize, EncoderError> {
        let aligned_offset = align::<ALIGN>(offset);
        let aligned_header_size = align::<ALIGN>(HEADER_SIZE);
        let required_size = aligned_offset + aligned_header_size;

        if buf.remaining_mut() < required_size {
            return Err(EncoderError::InsufficientSpaceForHeader {
                required: required_size,
                available: buf.remaining_mut(),
            });
        }

        Ok(aligned_offset)
    }

    #[inline]
    fn check_buffer_size_for_decoding<const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<usize, EncoderError> {
        let aligned_offset = align::<ALIGN>(offset);
        let aligned_header_size = align::<ALIGN>(HEADER_SIZE);
        let required_size = aligned_offset + aligned_header_size;

        if buf.remaining() < required_size {
            return Err(EncoderError::InsufficientSpaceForHeader {
                required: required_size,
                available: buf.remaining(),
            });
        }

        Ok(aligned_offset)
    }

    #[inline]
    unsafe fn decode_uninit<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, EncoderError> {
        let aligned_offset = Self::check_buffer_size_for_decoding::<ALIGN>(buf, offset)?;
        let mut value = MaybeUninit::uninit();
        Self::decode_uninit_inner::<B, ALIGN>(buf, aligned_offset, value.as_mut_ptr())?;
        Ok(value.assume_init())
    }

    unsafe fn decode_uninit_inner<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
        ptr: *mut Self,
    ) -> Result<(), EncoderError>;
}

#[inline]
pub const fn align<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}
