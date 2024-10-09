extern crate alloc;

use crate::{
    encoder::{align_up, read_u32_aligned, write_u32_aligned},
    error::{CodecError, DecodingError},
};
use byteorder::ByteOrder;
use bytes::{Buf, Bytes, BytesMut};
use core::mem;

/// Write bytes in Solidity compatible format
pub fn write_bytes_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &mut BytesMut,
    header_offset: usize,
    data: &[u8],
    elements: u32, // Number of elements
) -> usize {
    let aligned_offset = align_up::<ALIGN>(header_offset);

    // Ensure we have enough space to write the offset
    if buf.len() < aligned_offset {
        buf.resize(aligned_offset, 0);
    }
    let data_offset = buf.len();

    // Write length of the data (number of elements)
    write_u32_aligned::<B, ALIGN>(buf, data_offset, elements as u32);

    // Append the actual data
    buf.extend_from_slice(data);

    // Return the number of bytes written (including alignment)
    buf.len() - data_offset
}

// write bytes to the end of buffer
// old_buf_data...elements...data
pub fn write_bytes_solidity2<B: ByteOrder, const ALIGN: usize>(
    buf: &mut BytesMut,
    data: &[u8],
    elements: u32, // Number of elements
) -> usize {
    let data_offset = buf.len();
    // Write length of the data (number of elements)
    write_u32_aligned::<B, ALIGN>(buf, data_offset, elements as u32);

    // Append the actual data
    buf.extend_from_slice(data);

    // Return the number of bytes written (including alignment)
    buf.len() - data_offset
}

/// Write bytes in WASM compatible format
pub fn write_bytes_wasm<B: ByteOrder, const ALIGN: usize>(
    buf: &mut BytesMut,
    header_offset: usize,
    data: &[u8],
) -> usize {
    let aligned_offset = align_up::<ALIGN>(header_offset);
    let aligned_elem_size = align_up::<ALIGN>(mem::size_of::<u32>());
    let aligned_header_size = aligned_elem_size * 2;

    // Ensure we have enough space to write the header
    if buf.len() < aligned_offset + aligned_header_size {
        buf.resize(aligned_offset + aligned_header_size, 0);
    }

    // We append the data to the end of buffer
    let data_offset = buf.len();

    // Write offset and data size
    write_u32_aligned::<B, ALIGN>(buf, aligned_offset, data_offset as u32);
    write_u32_aligned::<B, ALIGN>(buf, aligned_offset + aligned_elem_size, data.len() as u32);

    // Append the actual data
    buf.extend_from_slice(data);

    // Return the number of bytes written (including alignment)
    buf.len() - data_offset
}

/// Universal function to write bytes in Solidity or WASM compatible format
pub fn write_bytes<B, const ALIGN: usize, const SOL_MODE: bool>(
    buf: &mut BytesMut,
    header_offset: usize,
    data: &[u8],
    elements: u32, // number of elements in dynamic array
) -> usize
where
    B: ByteOrder,
{
    match SOL_MODE {
        true => write_bytes_solidity::<B, ALIGN>(buf, header_offset, data, elements),
        false => write_bytes_wasm::<B, ALIGN>(buf, header_offset, data),
    }
}

pub fn read_bytes_header_wasm<B: ByteOrder, const ALIGN: usize>(
    buffer: &impl Buf,
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(mem::size_of::<u32>());

    if buffer.remaining() < aligned_offset + aligned_elem_size * 2 {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset + aligned_elem_size * 2,
            found: buffer.remaining(),
            msg: "buffer too small to read bytes header".to_string(),
        }));
    }

    let data_offset = read_u32_aligned::<B, ALIGN>(buffer, aligned_offset)? as usize;

    let data_len =
        read_u32_aligned::<B, ALIGN>(buffer, aligned_offset + aligned_elem_size)? as usize;

    Ok((data_offset, data_len))
}

// Reads the header of the bytes data in Solidity format
// and returns:
// - offset of the data (header not included). So real offset is offset + header_size
// - size of the data
pub fn read_bytes_header_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &impl Buf,
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    println!("~op.read_bytes_header_solidity");
    let aligned_offset = align_up::<ALIGN>(offset);

    let mut data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
    println!("Data offset: {}", data_offset);

    // If the data offset is 32, that means that actual length is the next word, otherwise we
    if data_offset != 32 {
        data_offset += 32;
    }

    let data_len = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;
    println!("Data length: {}", data_len);

    Ok((data_offset, data_len))
}

/// Buf should start from the beginning of the data
/// If we have a nested array, we need to provide buf without the header. It's crucial, because all
/// offsets sets for the nested arrays are relative to the beginning of the data let original:
/// Vec<Vec<u32>> = vec![vec![1, 2, 3], vec![4, 5]];
/// solidity

/// 000 000  : 00 00 00 20   ||  032 |
/// 032 000  : 00 00 00 02   ||  002 |

/// 000 000  : 00 00 00 40   ||  064 | <---- buf should start here
/// 032 032  : 00 00 00 c0   ||  192 |
/// 064 064  : 00 00 00 03   ||  003 |
/// 096 096  : 00 00 00 01   ||  001 |
/// 128 128  : 00 00 00 02   ||  002 |
/// 160 160  : 00 00 00 03   ||  003 |
/// 192 192  : 00 00 00 02   ||  002 |
/// 224 224  : 00 00 00 04   ||  004 |
/// 256 256  : 00 00 00 05   ||  005 |
pub fn read_bytes_header_solidity2<B: ByteOrder, const ALIGN: usize>(
    buf: &impl Buf,
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    println!("~op.read_bytes_header_solidity");
    let aligned_offset = align_up::<ALIGN>(offset);

    let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
    println!("Data offset: {}", data_offset);

    let data_len = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;
    println!("Data length: {}", data_len);

    Ok((data_offset, data_len))
}

/// Reads the header of the bytes data in Solidity or WASM compatible format
/// Returns the offset and size of the data
pub fn read_bytes_header<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>(
    buf: &impl Buf,
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    match SOL_MODE {
        true => read_bytes_header_solidity2::<B, ALIGN>(buf, offset),
        false => read_bytes_header_wasm::<B, ALIGN>(buf, offset),
    }
}

pub fn read_bytes_wasm<B: ByteOrder, const ALIGN: usize>(
    buf: &impl Buf,
    offset: usize,
) -> Result<Bytes, CodecError> {
    let (data_offset, data_len) = read_bytes_header_wasm::<B, ALIGN>(buf, offset)?;
    let data = buf.chunk()[data_offset..data_offset + data_len].to_vec();
    Ok(Bytes::from(data))
}

pub fn read_bytes<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>(
    buf: &impl Buf,
    offset: usize,
) -> Result<Bytes, CodecError> {
    let (data_offset, data_len) = read_bytes_header::<B, ALIGN, SOL_MODE>(buf, offset)?;
    println!("Data offset: {}, Data length: {}", data_offset, data_len);

    let data = if SOL_MODE {
        buf.chunk()[data_offset..data_offset + data_len].to_vec()
    } else {
        buf.chunk()[data_offset..].to_vec()
    };
    Ok(Bytes::from(data))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::encoder::{SolidityABI, WasmABI};
    use alloy_sol_types::{
        sol,
        sol_data::{self},
        SolType,
        SolValue,
    };
    use byteorder::{BigEndian, LE};
    #[test]
    fn test_write_bytes_sol() {
        let mut buf = BytesMut::new();

        // For byte slice
        let bytes: &[u8] = &[1, 2, 3, 4, 5];
        let written = write_bytes_solidity::<BigEndian, 32>(&mut buf, 0, bytes, bytes.len() as u32);
        assert_eq!(written, 37); // length (32) + (data + padding) (32)
        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 5, 1, 2, 3, 4, 5,
        ];

        assert_eq!(buf.to_vec(), expected);
        let mut buf = BytesMut::new();

        let offset = buf.len();

        // For Vec<u32>

        let vec_u32 = [0u8, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30];

        let written = write_bytes_solidity::<BigEndian, 32>(&mut buf, offset, &vec_u32, 3);
        assert_eq!(written, 44); // length (32) + data

        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 3, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30,
        ];
        assert_eq!(buf.to_vec(), expected);
    }

    #[test]
    fn test_read_bytes_header_solidity() {
        let original = alloy_primitives::Bytes::from(vec![1, 2, 3, 4, 5]);

        let mut buf = BytesMut::new();
        SolidityABI::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();

        let encoded_alloy = &sol_data::Bytes::abi_encode(&original)[..];

        println!("alloy encoded: {:?}", encoded_alloy);
        println!("encoded: {:?}", hex::encode(&encoded));
        let (offset, size) = read_bytes_header::<BigEndian, 32, true>(&encoded, 0).unwrap();

        println!("Offset: {}, Size: {}", offset, size);

        assert_eq!(offset, 32);
        assert_eq!(size, 5);
    }

    #[test]
    fn test_read_bytes_header_solidity2() {
        let original: Vec<Vec<u32>> = vec![vec![1, 2, 3], vec![4, 5]];

        let mut buf = BytesMut::new();
        SolidityABI::encode(&original, &mut buf, 0).unwrap();

        let encoded = buf.freeze();
        println!("encoded: {:?}", hex::encode(&encoded));

        let chunk = &encoded.chunk()[64..];

        // 1st vec
        let (offset, size) = read_bytes_header_solidity2::<BigEndian, 32>(&chunk, 0).unwrap();
        assert_eq!(offset, 64);
        assert_eq!(size, 3);

        // 2nd vec
        let (offset, size) = read_bytes_header_solidity2::<BigEndian, 32>(&chunk, 32).unwrap();
        assert_eq!(offset, 192);
        assert_eq!(size, 2);
    }

    #[test]
    fn test_read_bytes_header_wasm() {
        let original = alloy_primitives::Bytes::from(vec![1, 2, 3, 4, 5]);

        let mut buf = BytesMut::new();
        WasmABI::encode(&original, &mut buf, 0).unwrap();

        let (offset, size) = read_bytes_header::<LE, 4, false>(&buf, 0).unwrap();

        println!("Offset: {}, Size: {}", offset, size);

        assert_eq!(offset, 8);
        assert_eq!(size, 5);
    }
}
