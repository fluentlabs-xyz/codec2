use byteorder::ByteOrder;
use bytes::{Buf, Bytes, BytesMut};

use crate::{
    encoder::{align_up, read_u32_aligned, write_u32_aligned},
    error::{CodecError, DecodingError},
};

const DEFAULT_HEADER_ELEM_SIZE: usize = 4;

// Write bytes to the buf for Solidity mode
pub fn write_bytes_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &mut BytesMut,
    offset: usize,
    bytes: &[u8],
    vec_size: usize,
) -> usize {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);

    let data_offset = buf.len();
    // Resize the buffer if it is too small
    // offset in buf + data offset
    if buf.len() < aligned_offset + aligned_elem_size {
        buf.resize(aligned_offset + aligned_elem_size, 0);
    }
    // TODO: d1r1 fix this, we can do it better
    let vec_or_bytes_len = if vec_size > 0 { vec_size } else { bytes.len() };
    // Write length of the data
    write_u32_aligned::<B, ALIGN, true>(buf, data_offset, vec_or_bytes_len as u32);

    println!("Data Offset: {}, Data Length: {}", data_offset, bytes.len());
    println!("buf: {:?}", &buf.to_vec());

    // Append data to the buf
    buf.extend_from_slice(bytes);
    if buf.len() % ALIGN != 0 {
        let padding = ALIGN - (buf.len() % ALIGN);
        buf.resize(buf.len() + padding, 0);
    }

    8
}

// Write bytes to the buf for WASM mode
fn write_bytes_wasm<B: ByteOrder, const ALIGN: usize>(
    buf: &mut BytesMut,
    offset: usize,
    bytes: &[u8],
) -> usize {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);
    let aligned_header_size = aligned_elem_size * 2;

    if buf.len() < aligned_offset + aligned_header_size {
        buf.resize(aligned_offset + aligned_header_size, 0);
    }

    // Write header and length as described for WASM
    // Here you can customize how exactly WASM encoding should differ
    // Example: writing the offset and length of both arrays
    let data_offset = buf.len();
    write_u32_aligned::<B, ALIGN, false>(buf, aligned_offset, data_offset as u32);
    write_u32_aligned::<B, ALIGN, false>(
        buf,
        aligned_offset + aligned_elem_size,
        bytes.len() as u32,
    );

    // Append the actual data
    buf.extend_from_slice(bytes);

    aligned_header_size
}

// Read bytes from the buf for Solidity mode
fn read_bytes_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Bytes, CodecError> {
    let (data_offset, data_len) = read_bytes_header::<B, ALIGN, true>(buf, offset)?;
    println!("Data Offset: {}, Data Length: {}", data_offset, data_len);
    let data_offset = data_offset + ALIGN; // Skip data_len header element
    let data = buf.chunk()[data_offset..data_offset + data_len].to_vec();
    Ok(Bytes::from(data))
}

// Read bytes from the buf for WASM mode
fn read_bytes_wasm<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Bytes, CodecError> {
    let (data_offset, data_len) = read_bytes_header::<B, ALIGN, false>(buf, offset)?;
    let data = buf.chunk()[data_offset..data_offset + data_len].to_vec();
    Ok(Bytes::from(data))
}

// Read bytes header for Solidity mode
fn read_bytes_header_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);

    if buf.remaining() < aligned_offset + aligned_elem_size * 2 {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset + aligned_elem_size * 2,
            found: buf.remaining(),
            msg: "buf too small to read bytes header".to_string(),
        }));
    }

    // Read data offset and data length from the buf for Solidity ABI
    let data_offset = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset) as usize;
    let data_len =
        read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset + aligned_elem_size) as usize;

    Ok((data_offset, data_len))
}

// Read bytes header for WASM mode
fn read_bytes_header_wasm<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);

    if buf.remaining() < aligned_offset + aligned_elem_size * 2 {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset + aligned_elem_size * 2,
            found: buf.remaining(),
            msg: "buf too small to read bytes header".to_string(),
        }));
    }

    // Read data offset and data length from the buf for WASM encoding
    // Modify the logic here if the WASM format uses different offsets or length locations
    let data_offset = read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset) as usize;
    let data_len =
        read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset + aligned_elem_size) as usize;

    Ok((data_offset, data_len))
}

// Universal function to call depending on the mode (Solidity or WASM)
pub fn write_bytes<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &mut BytesMut,
    offset: usize,
    bytes: &[u8],
) -> usize {
    if SOLIDITY_COMP {
        write_bytes_solidity::<B, ALIGN>(buf, offset, bytes, 0)
    } else {
        write_bytes_wasm::<B, ALIGN>(buf, offset, bytes)
    }
}

// Universal function to call depending on the mode (Solidity or WASM)
pub fn read_bytes<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Bytes, CodecError> {
    if SOLIDITY_COMP {
        read_bytes_solidity::<B, ALIGN>(buf, offset)
    } else {
        read_bytes_wasm::<B, ALIGN>(buf, offset)
    }
}

// Universal function to call depending on the mode (Solidity or WASM)
pub fn read_bytes_header<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    if SOLIDITY_COMP {
        read_bytes_header_solidity::<B, ALIGN>(buf, offset)
    } else {
        read_bytes_header_wasm::<B, ALIGN>(buf, offset)
    }
}
