use std::usize;

use crate::encoder::{align_up, read_u32_aligned, write_u32_aligned, ByteOrderExt, Encoder};
use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

const DEFAULT_HEADER_ELEM_SIZE: usize = 4;

// Write bytes to the buffer
// Returns the size of the header
// To avoid resizing buffer, you can pre-allocate the buffer with the size of the header before calling this function
// The header contains the offset and length of the data
// The actual data is appended to the buffer, after the header
pub fn write_bytes<B: ByteOrderExt, const ALIGN: usize>(
    buffer: &mut BytesMut,
    offset: usize,
    bytes: &[u8],
) -> usize {
    let aligned_offset = align_up::<ALIGN>(offset);

    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);
    let aligned_header_size = aligned_elem_size * 2;

    if buffer.len() < aligned_offset + aligned_header_size {
        buffer.resize(aligned_offset + aligned_header_size, 0);
    }
    // We append the data to the buffer. So the offset of the data is the current length of the buffer
    let data_offset = buffer.len();

    // Write header
    write_u32_aligned::<B, ALIGN>(buffer, aligned_offset, data_offset as u32);

    // Write length of the data
    write_u32_aligned::<B, ALIGN>(
        buffer,
        aligned_offset + aligned_elem_size,
        bytes.len() as u32,
    );

    // Append data
    buffer.extend_from_slice(bytes);

    aligned_header_size
}

pub fn read_bytes<B: ByteOrderExt, const ALIGN: usize>(
    buffer: &mut impl Buf,
    offset: usize,
) -> Bytes {
    let (data_offset, data_len) = read_bytes_header::<B, ALIGN>(buffer, offset);

    let data = buffer.chunk()[data_offset..data_offset + data_len].to_vec();

    Bytes::from(data)
}

pub fn read_bytes_header<B: ByteOrderExt, const ALIGN: usize>(
    buffer: &mut impl Buf,
    offset: usize,
) -> (usize, usize) {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(DEFAULT_HEADER_ELEM_SIZE);

    let data_offset = read_u32_aligned::<B, ALIGN>(buffer, aligned_offset) as usize;
    let data_len =
        read_u32_aligned::<B, ALIGN>(buffer, aligned_offset + aligned_elem_size) as usize;

    (data_offset, data_len)
}

#[cfg(test)]
mod tests {
    use byteorder::BigEndian;

    use super::*;

    #[test]
    fn test_write_to_existing_buf2() {
        let existing_data = &[
            0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100,
        ];
        let mut buf = BytesMut::new();
        buf.extend_from_slice(existing_data);

        let original = Bytes::from_static(b"Hello, World");
        // Write the data to the buffer
        let _result = write_bytes::<BigEndian, 8>(&mut buf, 16, &original);

        let expected = [
            0, 0, 0, 0, 0, 0, 0, 32, //
            0, 0, 0, 0, 0, 0, 0, 12, //
            0, 0, 0, 0, 0, 0, 0, 44, //
            0, 0, 0, 0, 0, 0, 0, 12, //
            72, 101, 108, 108, 111, 44, 32, 87, //
            111, 114, 108, 100, 72, 101, 108, 108, //
            111, 44, 32, 87, 111, 114, 108, 100, //
        ];

        assert_eq!(buf.to_vec(), expected);

        let mut encoded = buf.freeze();

        let decoded = read_bytes::<BigEndian, 8>(&mut encoded, 0);

        assert_eq!(decoded, original);
    }
}
