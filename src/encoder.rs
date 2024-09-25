use std::{f32::consts::E, marker::PhantomData};

use byteorder::{ByteOrder, BE, LE};
use bytes::{Buf, Bytes, BytesMut};

use crate::error::CodecError;

// TODO: @d1r1 Investigate whether decoding the result into an uninitialized memory (e.g., using `MaybeUninit`)
// would be more efficient than initializing with `Default`.
// This could potentially reduce unnecessary memory initialization overhead in cases where
// the default value is not required before the actual decoding takes place.
// Consider benchmarking both approaches to measure performance differences.

pub trait Encoder: Sized {
    /// Header used to save metadata about the encoded value.
    const HEADER_SIZE: usize;

    /// How many bytes we should allocate for the encoded value.
    /// This is the sum of the header size and the known data size.
    fn size_hint<const ALIGN: usize>(&self) -> usize {
        align_up::<ALIGN>(Self::HEADER_SIZE)
    }

    /// Encodes the value into the given buf at the specified offset. The buf must be large enough to hold at least `align(offset) + Self::HEADER_SIZE` bytes.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buf to encode into.
    /// * `offset` - The offset in the buf to start encoding at.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encoding was successful, or an `EncoderError` if there was a problem.
    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError>;

    /// Decodes a value from the given buf starting at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buf to decode from.
    /// * `offset` - The offset in the buf to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns the decoded value if successful, or an `EncoderError` if there was a problem.
    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError>;

    /// Decodes the header to determine the size of the encoded data and offset to the data.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buf to decode from.
    /// * `offset` - The offset in the buf to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(offset, data_length)` if successful, or an `EncoderError` if there was a problem.
    ///
    /// For primitive types, the header size is 0, so the offset is returned as-is.
    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError>;
}

pub struct EncoderMode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool> {
    _byte_order: PhantomData<B>,
}

pub type SolidityEncoderMode = EncoderMode<BE, 32, true>;
pub type WasmEncoderMode = EncoderMode<LE, 4, false>;

pub struct EncoderModeAdapter<T, M>(PhantomData<(T, M)>);

impl<T, B, const ALIGN: usize, const SOLIDITY_COMP: bool>
    EncoderModeAdapter<T, EncoderMode<B, ALIGN, SOLIDITY_COMP>>
where
    T: Encoder,
    B: ByteOrder,
{
    pub fn encode(value: &T, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        value.encode::<B, ALIGN, SOLIDITY_COMP>(buf, offset)
    }

    pub fn decode(buf: &(impl Buf + ?Sized), offset: usize) -> Result<T, CodecError> {
        T::decode::<B, ALIGN, SOLIDITY_COMP>(buf, offset)
    }

    pub fn partial_decode(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        T::partial_decode::<B, ALIGN, SOLIDITY_COMP>(buf, offset)
    }

    pub fn size_hint(value: &T) -> usize {
        value.size_hint::<ALIGN>()
    }
}

/// Example usage:
//
// use crate::encoder::{SolidityABI, WasmABI};
// WasmABI::<u32>::encode(&42, &mut buf, 0);
// let value = WasmABI::<u32>::decode(&buf, 0);
//
//
pub type SolidityABI<T> = EncoderModeAdapter<T, SolidityEncoderMode>;
pub type WasmABI<T> = EncoderModeAdapter<T, WasmEncoderMode>;

// TODO: move functions bellow to the utils module

// TODO: d1r1 is it possible to make this fn const?
pub fn is_big_endian<B: ByteOrder>() -> bool {
    B::read_u16(&[0x12, 0x34]) == 0x1234
}

/// Rounds up the given offset to the nearest multiple of ALIGN.
/// ALIGN must be a power of two.
#[inline]
pub const fn align_up<const ALIGN: usize>(offset: usize) -> usize {
    (offset + ALIGN - 1) & !(ALIGN - 1)
}

/// Aligns the source bytes to the specified alignment.
pub fn align<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(src: &[u8]) -> Bytes {
    let aligned_src_len = align_up::<ALIGN>(src.len());
    let aligned_total_size = aligned_src_len.max(ALIGN);
    let mut aligned = BytesMut::zeroed(aligned_total_size);

    if is_big_endian::<B>() {
        // For big-endian, copy to the end of the aligned array
        let start = aligned_total_size - src.len();
        aligned[start..].copy_from_slice(src);
    } else {
        // For little-endian, copy to the start of the aligned array
        aligned[..src.len()].copy_from_slice(src);
    }

    aligned.freeze()
}

pub fn write_u32_aligned<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &mut BytesMut,
    offset: usize,
    value: u32,
) {
    let aligned_value_size = align_up::<ALIGN>(4);

    if buf.len() < offset + aligned_value_size {
        buf.resize(offset + aligned_value_size, 0);
    }

    if is_big_endian::<B>() {
        // For big-endian, copy to the end of the aligned array
        let start = offset + aligned_value_size - 4;
        B::write_u32(&mut buf[start..], value);
    } else {
        // For little-endian, copy to the start of the aligned array
        B::write_u32(&mut buf[offset..offset + 4], value);
    }
}

pub fn read_u32_aligned<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<u32, CodecError> {
    let aligned_value_size = align_up::<ALIGN>(4);

    // TODO: "add overflow check"

    if is_big_endian::<B>() {
        // For big-endian, copy from the end of the aligned array
        let start = offset + aligned_value_size - 4;
        if buf.remaining() < start + 4 {
            return Err(CodecError::Decoding(
                crate::error::DecodingError::BufferTooSmall {
                    expected: start + 4,
                    found: buf.remaining(),
                    msg: "failed to aligned read u32".to_string(),
                },
            ));
        }

        Ok(B::read_u32(&buf.chunk()[start..start + 4]))
    } else {
        // For little-endian, copy from the start of the aligned array
        Ok(B::read_u32(&buf.chunk()[offset..offset + 4]))
    }
}
