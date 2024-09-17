use core::{mem, ptr};

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use bytes::{buf, BufMut, Bytes, BytesMut};

use crate::{encoder::EncoderError, utils::print_buffer_debug};

/// Writes a slice to an aligned position in a buffer.
///
/// # Safety
///
/// This function is unsafe because it performs unchecked writes to memory.
/// The caller must ensure that the buffer has enough capacity to accommodate the aligned write.
///
/// # Arguments
///
/// * `dest_buf`: The destination buffer to write to.
/// * `write_offset`: The initial offset in the buffer where writing should start.
/// * `src_slice`: The source slice to be written.
/// * `alignment`: The alignment boundary (in bytes) for the write operation.
/// * `write_position`: Specifies whether to write at the start or end of the aligned space.
///
/// # Returns
///
/// `Result<(), EncoderError>` - Ok if the write was successful, Err otherwise.
#[inline(always)]
pub unsafe fn write_slice_aligned<const ALIGN: usize>(
    dest_buf: &mut impl BufMut,
    write_offset: usize,
    src_slice: &[u8],
    write_position: &WritePosition,
) -> Result<(), EncoderError> {
    // Align the buffer offset
    let aligned_offset = align_offset::<ALIGN>(write_offset);

    // Calculate the size of the value and the word size
    let src_size = src_slice.len();
    let word_size = ALIGN;

    // Ensure the buffer has enough space for the operation
    let required_space = word_size.max(src_size);

    if dest_buf.remaining_mut() < aligned_offset + required_space {
        return Err(EncoderError::BufferTooSmall {
            required: required_space,
            available: dest_buf.remaining_mut(),
            msg: "Not enough space in buffer for aligned offset + required_space".to_string(),
        });
    }

    // Advance the buffer to the aligned offset
    dest_buf.advance_mut(aligned_offset);

    // Get mutable access to the uninitialized chunk
    let chunk = dest_buf.chunk_mut();

    let chunk_len = chunk.len();
    if chunk_len < required_space {
        return Err(EncoderError::BufferTooSmall {
            required: required_space,
            available: chunk_len,
            msg: "Not enough space in chunk".to_string(),
        });
    }

    // Get pointer to the aligned position
    let mut write_ptr = chunk.as_mut_ptr();

    // Write the value according to the specified position
    match write_position {
        WritePosition::End => {
            // Write from the end of the aligned space
            write_ptr = write_ptr.add(word_size - src_size); // Move pointer to the correct position
        }
        WritePosition::Start => {
            // Write from the start of the aligned space (no change to write_ptr needed)
        }
    }

    ptr::copy_nonoverlapping(src_slice.as_ptr(), write_ptr, src_size);

    // Advance the buffer to reflect the written data
    dest_buf.advance_mut(word_size);

    Ok(())
}

/// Specifies the position to write the value within the aligned space.
pub enum WritePosition {
    /// Write the value at the start of the aligned space (used for little-endian)
    Start,
    /// Write the value at the end of the aligned space (used for big-endian)
    End,
}
/// Aligns the given offset to the specified boundary.
#[inline(always)]
fn align_offset<const ALIGN: usize>(offset: usize) -> usize {
    (offset + (ALIGN - 1)) & !(ALIGN - 1)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Bytes;
    use byteorder::{BigEndian, LittleEndian};
    use bytes::BytesMut;

    use crate::utils::print_buffer_debug;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_aligned_write_little_endian() {
        let mut buffer = BytesMut::with_capacity(16);

        let value: u32 = 0x12345678;

        let value_le = value.to_le_bytes();
        println!(">>> value_le: {:?}", value_le);

        print_buffer_debug(&buffer, 0);
        unsafe {
            write_slice_aligned::<8>(&mut buffer, 8, &value_le, &WritePosition::Start).unwrap();
        }

        print_buffer_debug(&buffer, 0);
        assert_eq!(
            buffer,
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0x78, 0x56, 0x34, 0x12, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_aligned_write_big_endian() {
        let mut buffer = BytesMut::with_capacity(32);
        buffer.put_u32(1);

        let value: u32 = 0x12345678;

        unsafe {
            write_slice_aligned::<8>(&mut buffer, 4, &value.to_be_bytes(), &WritePosition::End)
                .unwrap();
        }

        print_buffer_debug(&BytesMut::from(&buffer[..]), 0);

        assert_eq!(
            buffer.to_vec(),
            vec![0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x12, 0x34, 0x56, 0x78]
        );
    }
}
