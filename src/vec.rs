extern crate alloc;
use alloc::vec::Vec;

use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

use crate::bytes::write_bytes_solidity;
use crate::error::{CodecError, DecodingError};
use crate::{
    bytes::{read_bytes, read_bytes_header, write_bytes},
    encoder::{align_up, read_u32_aligned, write_u32_aligned, Encoder},
};

/// We encode dynamic arrays as following:
/// - header
/// - + length - number of elements inside vector
/// - + offset - offset inside structure
/// - + size - number of encoded bytes
/// - body
/// - + raw bytes of the vector
///
///
/// For solidity we don't have size.
/// - header
/// - + offset
/// - + length
/// - body
/// - + raw bytes of the vector
impl<T: Default + Sized + Encoder + std::fmt::Debug> Encoder for Vec<T> {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        if SOLIDITY_COMP {
            encode_vector_solidity::<B, T, ALIGN>(self, buf, offset, self.len())
        } else {
            encode_vector_wasm::<B, T, ALIGN>(self, buf, offset)
        }
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        if SOLIDITY_COMP {
            decode_vec_solidity::<B, T, ALIGN>(buf, offset)
        } else {
            decode_vec_wasm::<B, T, ALIGN>(buf, offset)
        }
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let elem_size = align_up::<ALIGN>(4);

        if buf.remaining() < aligned_offset + elem_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + elem_size,
                found: buf.remaining(),
                msg: "failed to decode vector length".to_string(),
            }));
        }

        read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset)
    }
}

// vec![vec![1,2], vec![3,4,5]]
// Offset  | Data
// --------------------------------------------
// 0       | Смещение до данных (32 байта) -> 32
// 32      | Длина внешнего массива -> 2 (два вложенных массива)
// 64      | Смещение до первого вложенного массива -> 64
// 96      | Смещение до второго вложенного массива -> 160

// 128     | Длина первого вложенного массива -> 2 (два элемента)
// 160     | Первый элемент первого массива -> 1
// 192     | Второй элемент первого массива -> 2

// 256     | Длина второго вложенного массива -> 3 (три элемента)
// 288     | Первый элемент второго массива -> 3
// 320     | Второй элемент второго массива -> 4
// 352     | Третий элемент второго массива -> 5

// основное отличие bytes от vec -

fn encode_vector_solidity<
    B: ByteOrder,
    T: Default + Sized + Encoder + std::fmt::Debug,
    const ALIGN: usize,
>(
    vec: &Vec<T>,
    buf: &mut BytesMut,
    offset: usize,
    vec_size: usize,
) -> Result<(), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(4);

    println!("op encode_vector_solidity");
    println!("vec: {:?}", &vec);
    // println!("buf start: {:?}", &buf.to_vec());
    println!("offset: {}", offset);
    println!("buf len: {}", buf.len());
    println!("aligned_offset: {}", aligned_offset);
    // Check if we can store offset
    if buf.len() < aligned_offset + aligned_elem_size {
        buf.resize(aligned_offset + aligned_elem_size, 0);
    }

    let data_offset = buf.len();
    // Write offset

    write_u32_aligned::<B, ALIGN, true>(buf, aligned_offset, data_offset as u32);

    // // Encode values
    let mut value_encoder = BytesMut::zeroed(ALIGN.max(T::HEADER_SIZE) * vec.len());

    for (index, obj) in vec.iter().enumerate() {
        let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;
        obj.encode::<B, ALIGN, true>(&mut value_encoder, elem_offset)
            .expect("Failed to encode vector element");
    }

    write_bytes_solidity::<B, ALIGN>(
        buf,
        aligned_offset + aligned_elem_size,
        &value_encoder.freeze(),
        vec_size,
    );

    Ok(())
}

fn encode_vector_wasm<
    B: ByteOrder,
    T: Default + Sized + Encoder + std::fmt::Debug,
    const ALIGN: usize,
>(
    vec: &Vec<T>,
    buf: &mut BytesMut,
    offset: usize,
) -> Result<(), CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_elem_size = align_up::<ALIGN>(4);
    let aligned_header_size = align_up::<ALIGN>(Vec::<T>::HEADER_SIZE);

    // Check if we can store header
    if buf.len() < aligned_offset + aligned_header_size {
        buf.resize(aligned_offset + aligned_elem_size * 3, 0);
    }

    // Write length
    write_u32_aligned::<B, ALIGN, false>(buf, aligned_offset, vec.len() as u32);

    // If vector is empty, we don't need to encode anything
    if vec.is_empty() {
        write_u32_aligned::<B, ALIGN, false>(
            buf,
            aligned_offset + aligned_elem_size,
            aligned_header_size as u32,
        );
        write_u32_aligned::<B, ALIGN, false>(buf, aligned_offset + aligned_elem_size * 2, 0);
        return Ok(());
    }

    // Encode values
    // First we reserve space to store headers for each element
    let mut value_encoder = BytesMut::zeroed(ALIGN.max(T::HEADER_SIZE) * vec.len());

    // Then we encode each element and store header on the specific offset, and actual data written to the end
    for (index, obj) in vec.iter().enumerate() {
        let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;
        obj.encode::<B, ALIGN, false>(&mut value_encoder, elem_offset)
            .expect("Failed to encode vector element");
    }
    // Write values right after the offset
    write_bytes::<B, ALIGN, false>(
        buf,
        aligned_offset + aligned_elem_size,
        &value_encoder.freeze(),
    );

    Ok(())
}

fn decode_vec_wasm<
    B: ByteOrder,
    T: Default + Sized + Encoder + std::fmt::Debug,
    const ALIGN: usize,
>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Vec<T>, CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_header_el_size = align_up::<ALIGN>(4);

    if buf.remaining() < aligned_offset + aligned_header_el_size {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset + aligned_header_el_size,
            found: buf.remaining(),
            msg: "failed to decode vector length".to_string(),
        }));
    }

    let data_len = read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset) as usize;
    if data_len == 0 {
        return Ok(Vec::new());
    }

    let input_bytes = read_bytes::<B, ALIGN, false>(buf, aligned_offset + aligned_header_el_size)?;

    decode_elements::<B, T, ALIGN, false>(&input_bytes, data_len)
}

fn decode_vec_solidity<
    B: ByteOrder,
    T: Default + Sized + Encoder + std::fmt::Debug,
    const ALIGN: usize,
>(
    buf: &(impl Buf + ?Sized),
    offset: usize,
) -> Result<Vec<T>, CodecError> {
    let aligned_offset = align_up::<ALIGN>(offset);
    let aligned_header_el_size = align_up::<ALIGN>(4);

    if buf.remaining() < aligned_offset {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: aligned_offset,
            found: buf.remaining(),
            msg: "failed to decode vector offset".to_string(),
        }));
    }

    let data_offset = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset) as usize;
    if buf.remaining() < data_offset + aligned_header_el_size {
        return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
            expected: data_offset + aligned_header_el_size,
            found: buf.remaining(),
            msg: "failed to decode vector length".to_string(),
        }));
    }

    let data_len = read_u32_aligned::<B, ALIGN, true>(buf, data_offset) as usize;
    if data_len == 0 {
        return Ok(Vec::new());
    }

    let input_bytes = read_bytes::<B, ALIGN, true>(buf, data_offset + aligned_header_el_size)?;

    decode_elements::<B, T, ALIGN, true>(&input_bytes, data_len)
}

fn decode_elements<
    B: ByteOrder,
    T: Default + Sized + Encoder + std::fmt::Debug,
    const ALIGN: usize,
    const SOLIDITY_COMP: bool,
>(
    input_bytes: &[u8],
    data_len: usize,
) -> Result<Vec<T>, CodecError> {
    let mut result = Vec::with_capacity(data_len);
    let val_size = align_up::<ALIGN>(T::HEADER_SIZE);

    for i in 0..data_len {
        let elem_offset = i * val_size;
        let mut input_bytes = input_bytes;
        let value = T::decode::<B, ALIGN, SOLIDITY_COMP>(&mut input_bytes, elem_offset)?;
        result.push(value);
    }

    Ok(result)
}
#[cfg(test)]
mod tests {
    use crate::encoder::Encoder;
    use byteorder::{BigEndian, LittleEndian};
    use bytes::{Bytes, BytesMut};

    use super::*;

    #[test]
    fn test_empty_vec_u32() {
        let original: Vec<u32> = Vec::new();
        let mut buf = BytesMut::new();

        original
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();
        let mut encoded = buf.freeze();
        let expected = hex::decode("000000000c00000000000000").expect("Failed to decode hex");
        assert_eq!(encoded, Bytes::from(expected));

        let decoded = Vec::<u32>::decode::<LittleEndian, 4, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32() {
        let original: Vec<u32> = vec![1, 2, 3, 4];
        let mut buf = BytesMut::new();

        original.encode::<BigEndian, 4, false>(&mut buf, 0).unwrap();
        let mut encoded = buf.freeze();

        let expected_encoded = "000000040000000c0000001000000001000000020000000300000004";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        println!("{:?}", hex::encode(&encoded));

        let decoded = <Vec<u32>>::decode::<BigEndian, 4, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32_with_offset() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        original
            .encode::<LittleEndian, 4, false>(&mut buf, 3)
            .unwrap();
        let mut encoded = buf.freeze();
        println!("{:?}", hex::encode(&encoded));

        let decoded = Vec::<u32>::decode::<LittleEndian, 4, false>(&mut encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_vec_u8_with_offset() {
        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        original
            .encode::<LittleEndian, 4, false>(&mut buf, 3)
            .unwrap();
        let mut encoded = buf.freeze();
        println!("{:?}", hex::encode(&encoded));

        let decoded: Vec<u8> =
            Vec::<u8>::decode::<LittleEndian, 4, false>(&mut encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        original
            .encode::<LittleEndian, 2, false>(&mut buf, 0)
            .unwrap();
        let mut encoded = buf.freeze();
        println!("{:?}", hex::encode(&encoded));
        let expected_encoded = "020000000c00000022000000020000001800000004000000030000001c0000000600000003000400050006000700";

        assert_eq!(hex::encode(&encoded), expected_encoded);

        let decoded = Vec::<Vec<u16>>::decode::<LittleEndian, 2, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_nested_vec_a4_le() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        original
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();
        let mut encoded = buf.freeze();
        let decoded = Vec::<Vec<u16>>::decode::<LittleEndian, 4, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_nested_vec_a4_be() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        original.encode::<BigEndian, 4, false>(&mut buf, 0).unwrap();
        let mut encoded = buf.freeze();

        let decoded = Vec::<Vec<u16>>::decode::<BigEndian, 4, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_large_vec() {
        let original: Vec<u64> = (0..1000).collect();
        let mut buf = BytesMut::new();

        original.encode::<BigEndian, 8, false>(&mut buf, 0).unwrap();
        let mut encoded = buf.freeze();

        let decoded = Vec::<u64>::decode::<BigEndian, 8, false>(&mut encoded, 0).unwrap();

        assert_eq!(original, decoded);
    }
}
