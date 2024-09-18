use byteorder::{BigEndian, ByteOrder, LittleEndian};
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

    /// Encodes the value into the given buffer at the specified offset. The buffer must be large enough to hold at least `align(offset) + Self::HEADER_SIZE` bytes.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to encode into.
    /// * `offset` - The offset in the buffer to start encoding at.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if encoding was successful, or an `EncoderError` if there was a problem.
    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError>;

    /// Decodes a value from the given buffer starting at the specified offset.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns the decoded value if successful, or an `EncoderError` if there was a problem.
    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Self, CodecError>;

    /// Decodes the header to determine the size of the encoded data and offset to the data.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to decode from.
    /// * `offset` - The offset in the buffer to start decoding from.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(offset, data_length)` if successful, or an `EncoderError` if there was a problem.
    ///
    /// For primitive types, the header size is 0, so the offset is returned as-is.
    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError>;
}

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
    buffer: &mut BytesMut,
    offset: usize,
    value: u32,
) {
    let aligned_value_size = align_up::<ALIGN>(4);

    if buffer.len() < offset + aligned_value_size {
        buffer.resize(offset + aligned_value_size, 0);
    }

    if is_big_endian::<B>() {
        // For big-endian, copy to the end of the aligned array
        let start = offset + aligned_value_size - 4;
        B::write_u32(&mut buffer[start..], value);
    } else {
        // For little-endian, copy to the start of the aligned array
        B::write_u32(&mut buffer[offset..offset + 4], value);
    }
}

pub fn read_u32_aligned<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buffer: &impl Buf,
    offset: usize,
) -> u32 {
    let aligned_value_size = align_up::<ALIGN>(4);

    if is_big_endian::<B>() {
        // For big-endian, copy from the end of the aligned array
        let start = offset + aligned_value_size - 4;
        B::read_u32(&buffer.chunk()[start..])
    } else {
        // For little-endian, copy from the start of the aligned array
        B::read_u32(&buffer.chunk()[offset..offset + 4])
    }
}
