extern crate alloc;
use alloc::vec::Vec;
use byteorder::ByteOrder;
use bytes::{Buf, Bytes, BytesMut};
use core::{marker::PhantomData, mem};

use crate::{
    encoder::{read_u32_aligned, write_u32_aligned},
    error::{CodecError, DecodingError},
};

/// Universal function to write bytes in Solidity or WASM compatible format
pub fn write_bytes<B, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &mut BytesMut,
    header_offset: usize,
    data: &[u8],
    elements: u32, // Size of data in bytes OR number of elements (if SOLIDITY_COMP)
) -> usize
where
    B: ByteOrder,
{
    let aligned_offset = align_up::<ALIGN>(header_offset);
    let aligned_elem_size = align_up::<ALIGN>(mem::size_of::<u32>());

    let aligned_header_size = if SOLIDITY_COMP {
        aligned_elem_size
    } else {
        aligned_elem_size * 2
    };

    let mut data_offset = 0;
    // Ensure we have enough space to write the offset
    if SOLIDITY_COMP {
        if buf.len() < aligned_offset {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }
        data_offset = buf.len();
        // Solidity mode: write data length only (length  - elements count, size - bytes count)

        write_u32_aligned::<B, ALIGN, true>(buf, data_offset, elements as u32);
    } else {
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }
        data_offset = buf.len();

        // WASM mode: write offset and data size
        write_u32_aligned::<B, ALIGN, false>(buf, aligned_offset, data_offset as u32);
        write_u32_aligned::<B, ALIGN, false>(
            buf,
            aligned_offset + aligned_elem_size,
            elements as u32,
        );
    }

    // Append the actual data
    buf.extend_from_slice(&data);

    // Return the number of bytes written (including alignment)
    buf.len() - data_offset
}

pub fn read_bytes_header_wasm<B: ByteOrder, const ALIGN: usize>(
    buffer: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(4);
    if buffer.remaining() < aligned_offset + aligned_elem_size * 2 {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset + aligned_elem_size * 2,
            found: buffer.remaining(),
            msg: "buffer too small to read bytes header".to_string(),
        }));
    }

    let data_offset = read_u32_aligned::<B, ALIGN, false>(buffer, aligned_offset)? as usize;
    let data_len =
        read_u32_aligned::<B, ALIGN, false>(buffer, aligned_offset + data_offset)? as usize;

    Ok((data_offset, data_len))
}

/// Reads the header of the bytes data in Solidity or WASM compatible format
/// Returns the offset and size of the data
pub fn read_bytes_header<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<(usize, usize), CodecError> {

    if SOLIDITY_COMP {
        return read_bytes_header_solidity::<B, ALIGN>(buf, offset);
    }

    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_header_el_size = align_up::<ALIGN>(mem::size_of::<u32>());

    let header_size = if SOLIDITY_COMP {
        aligned_header_el_size
    } else {
        aligned_header_el_size * 2
    };
    println!("Header Size: {}", header_size);

    if buf.remaining() < aligned_offset + header_size {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset + header_size,
            found: buf.remaining(),
            msg: if SOLIDITY_COMP {
                "buffer too small to read bytes header for Solidity"
            } else {
                "buffer too small to read bytes header for WASM"
            }
            .to_string(),
        }));
    }

    if SOLIDITY_COMP {
        println!(">>> cursor: data_offset: {:?}", aligned_offset);
        // Solidity mode: read data length only (length  - elements count, size - bytes count)
        let data_offset = aligned_offset;
        let data_len = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset)? as usize;
        println!(
            "---->>> Data Offset: {}, Data Length: {}",
            data_offset, data_len
        );

        Ok((data_offset, data_len))
    } else {
        let data_offset = read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset)? as usize;
        let data_len =
            read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset + aligned_header_el_size)?
                as usize;

        Ok((data_offset, data_len))
    }
}

pub fn read_bytes_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Bytes, CodecError> {
    let (data_offset, data_len) = read_bytes_header::<B, ALIGN, true>(buf, offset)?;
    println!(">>>Data Offset: {}, Data Length: {}", data_offset, data_len);
    println!(
        ">>>111 Buf : {:?}",
        &buf.chunk()[data_offset..data_offset + data_len * 32]
    );

    // let data_offset = data_offset + ALIGN; // Skip data_len header element
    let data = buf.chunk()[data_offset..data_offset + data_len * 32].to_vec();
    Ok(Bytes::from(data))
}

pub fn read_bytes_solidity2<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Bytes, CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);

    // Read the length of the bytes
    let data_len = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset)? as usize;
    println!(">>>Data Length: {}", data_len);

    // The actual data starts after the length field
    let data_offset = aligned_offset + ALIGN;

    if buf.remaining() < data_offset + data_len {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: data_offset + data_len,
            found: buf.remaining(),
            msg: "buffer too small to read bytes data".to_string(),
        }));
    }

    let data = buf.chunk()[data_offset..data_offset + data_len].to_vec();
    Ok(Bytes::from(data))
}

fn read_bytes_header_solidity<B: ByteOrder, const ALIGN: usize>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<(usize, usize), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);

    let data_offset = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset)? as usize;
    let element_offset = data_offset + ALIGN;
    let element_len = read_u32_aligned::<B, ALIGN, true>(buf, data_offset)? as usize;

    Ok((element_offset, element_len))
}

pub fn read_bytes<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
    element_size: usize,
) -> Result<Bytes, CodecError> {
    println!("op read_bytes");
    println!(">>>123 Reading bytes at offset: {}", offset);
    println!(">>>123 elements_size: {}", element_size);
    let (data_offset, elements_count) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(buf, offset)?;
    println!(
        ">>>123 SOLIDITY? {} Data Offset: {}, Elements count: {}",
        SOLIDITY_COMP, data_offset, elements_count
    );
    if elements_count == 0 {
        return Ok(Bytes::new());
    }

    let actual_data_offset = if SOLIDITY_COMP {
        data_offset + ALIGN // Skip element_count header in Solidity mode
    } else {
        data_offset
    };

    println!(">>>123 Actual Data Offset: {}", actual_data_offset);
    println!(">>>123 elements_count: {}", elements_count);
    println!(">>>123 buf: {:?}", &buf.chunk()[actual_data_offset..]);

    let data_size = if SOLIDITY_COMP {
        elements_count * element_size
    } else {
        elements_count
    };

    println!(">>>123 Data Size: {}", data_size);

    if buf.remaining() < actual_data_offset + data_size {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: actual_data_offset + data_size,
            found: buf.remaining(),
            msg: "buffer too small to read data :(".to_string(),
        }));
    }

    let data = buf.chunk()[actual_data_offset..].to_vec();
    Ok(Bytes::from(data))
}

pub fn read_bytes_solidity_nested<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Bytes, CodecError> {
    let data_len = read_u32_aligned::<B, ALIGN, true>(buf, offset)? as usize;

    let data_offset = offset + ALIGN;

    if buf.remaining() < data_offset + data_len {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: data_offset + data_len,
            found: buf.remaining(),
            msg: "buffer too small to read bytes data".to_string(),
        }));
    }

    let data = buf.chunk()[data_offset..data_offset + data_len].to_vec();
    Ok(Bytes::from(data))
}

// Helper function for alignment
const fn align_up<const ALIGN: usize>(value: usize) -> usize {
    (value + (ALIGN - 1)) & !(ALIGN - 1)
}

#[cfg(test)]
mod tests {

    use super::*;
    use byteorder::{BigEndian, LittleEndian};
    use bytes::buf;

    #[test]
    fn test_write_bytes() {
        let mut buf = BytesMut::new();

        // For byte slice
        let bytes: &[u8] = &[1, 2, 3, 4, 5];
        let written = write_bytes::<BigEndian, 32, true>(&mut buf, 0, bytes, bytes.len() as u32);
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

        let written = write_bytes::<BigEndian, 32, true>(&mut buf, offset, &vec_u32, 3);
        assert_eq!(written, 44); // length (32) + data

        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 3, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30,
        ];
        assert_eq!(buf.to_vec(), expected);
    }

    #[test]
    fn test_read_bytes_header_solidity() {
        let mut buf = BytesMut::new();
        let data = [1u8, 2, 3, 4, 5];
        let written = write_bytes::<BigEndian, 32, true>(&mut buf, 0, &data, data.len() as u32);
        assert_eq!(written, 32 + 5); // header + data

        println!("Buffer>>>: {:?}", &buf.chunk()[..]);
        let (offset, size) = read_bytes_header::<BigEndian, 32, true>(&buf, 0).unwrap();

        println!("Offset: {}, Size: {}", offset, size);
        assert_eq!(offset, 0);
        assert_eq!(size, 5);

        let data: &[u8] = &[1, 2, 3, 4, 5, 6, 7];
        let mut buf = BytesMut::new();

        let written = write_bytes::<BigEndian, 32, true>(&mut buf, 0, data, data.len() as u32);
        assert_eq!(written, 39); // length (32) + data (5)

        let (offset, size) = read_bytes_header::<BigEndian, 32, true>(&buf, 0).unwrap();
        println!("Offset: {}, Size: {}", offset, size);

        assert_eq!(offset, 0);
        assert_eq!(size, 7);
    }

    #[test]
    fn test_read_bytes_header_wasm() {
        let mut buf = BytesMut::new();

        let data = [0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30];
        write_bytes::<LittleEndian, 4, false>(&mut buf, 0, &data, 3 as u32);

        let (data_offset, data_len) = read_bytes_header::<LittleEndian, 4, false>(&buf, 0).unwrap();

        assert_eq!(data_offset, 8);
        assert_eq!(data_len, 3); // 3 elements
    }

    #[test]
    fn test_write_read_bytes_u8_solidity() {
        let original_data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();

        // Encode
        let written = write_bytes::<BigEndian, 32, true>(&mut buf, 0, &original_data, 5 as u32);
        assert_eq!(written, 37); // 32 (header) + 5 (data)

        // Decode
        let decoded_data = read_bytes::<BigEndian, 32, true>(&buf, 0, 1).unwrap();

        assert_eq!(original_data, decoded_data);
    }
    // #[test]
    // fn test_write_read_vec_u32_solidity() {
    //     let original_bytes = [0u8, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5];
    //     let mut buf = BytesMut::new();

    //     // Encode
    //     let written = write_bytes::<BigEndian, 32, true>(&mut buf, 0, &original_bytes, 5 as u32);
    //     assert_eq!(written, 64); // 32 (header) + 32 (data + padding)

    //     // Decode
    //     let decoded = read_bytes::<BigEndian, u32, 32, true>(&buf, 0).unwrap();

    //     assert_eq!(&original_bytes, &decoded);
    // }
}
