extern crate alloc;
use alloc::vec::Vec;

use byteorder::ByteOrder;
use bytes::{Buf, BytesMut};

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
/// - body
/// - + length
/// - + raw bytes of the vector
impl<T: Default + Sized + Encoder + std::fmt::Debug> Encoder for Vec<T> {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(4);

        // For solidity we need to reserve space only for the offset
        let aligned_header_size = if SOLIDITY_COMP {
            aligned_elem_size
        } else {
            // For wasm we need to reserve space for offset, length and size
            aligned_elem_size * 3
        };
        // println!("Aligned header size: {:?}", aligned_header_size);
        // println!("Aligned elem size: {:?}", aligned_elem_size);
        // println!("Aligned offset: {:?}", aligned_offset);
        // println!(".1 buf {:?}", &buf.chunk()[..]);
        // Check if we can store header
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }
        // println!(".2 buf {:?}", &buf.chunk()[..]);
        // println!(".2.1 buf len: {:?}",  buf.len());

        if SOLIDITY_COMP {
            // Solidity mode: write offset only (current buffer length)
            write_u32_aligned::<B, ALIGN, true>(buf, aligned_offset, buf.len() as u32);
            // println!(".3 buf {:?}", &buf.chunk()[..]);
        } else {
            // WASM mode: write length only.
            write_u32_aligned::<B, ALIGN, false>(buf, aligned_offset, self.len() as u32);
        }

        if self.is_empty() {
            if SOLIDITY_COMP {
                write_u32_aligned::<B, ALIGN, true>(buf, buf.len(), 0);
            } else {
                write_u32_aligned::<B, ALIGN, false>(
                    buf,
                    aligned_offset + aligned_elem_size,
                    aligned_header_size as u32,
                );
                write_u32_aligned::<B, ALIGN, false>(
                    buf,
                    aligned_offset + aligned_elem_size * 2,
                    0,
                );
            }

            return Ok(());
        }

        // println!(".4 buf {:?}", &buf.chunk()[..]);

        let header_size = if SOLIDITY_COMP { 4 } else { T::HEADER_SIZE };
        // Encode values
        let mut value_encoder = BytesMut::zeroed(align_up::<ALIGN>(header_size) * self.len());
        println!(".333 cursor {:?}", &value_encoder.chunk().len());
        println!(".4 value encoder len: {:?}", value_encoder.len());
        println!("5. value encoder buf {:?}", &value_encoder.chunk()[..]);

        for (index, obj) in self.iter().enumerate() {
            let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;

            println!(
                ".6. elem offset {}; {} buf {:?}",
                elem_offset,
                index,
                &buf.chunk()[..]
            );

            obj.encode::<B, ALIGN, SOLIDITY_COMP>(&mut value_encoder, elem_offset)
                .expect("Failed to encode vector element");
            println!(".7.{} buf {:?}", index, &buf.chunk()[..]);
        }

        let data = value_encoder.freeze();

        // We need to provide vector size for solidity, because we can't calculate it from the data itself. For wasm we write bytes size of the data instead of elements count, so we can provide data size only.
        let elements = if SOLIDITY_COMP {
            self.len()
        } else {
            data.len()
        } as u32;

        write_bytes::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_elem_size,
            &data,
            elements,
        );
        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        println!("op decode vec");
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_el_size = align_up::<ALIGN>(4);
        let val_size = ALIGN.max(T::HEADER_SIZE);

        let (data_offset, data_bytes_len) =
            Self::partial_decode::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset)?;

        println!(")()(Data offset: {:?}", data_offset);
        println!(")()(Data bytes len: {:?}", data_bytes_len);
        let data_len = if SOLIDITY_COMP {
            data_bytes_len
        } else {
            read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset)? as usize
        };

        if data_len == 0 {
            return Ok(Vec::new());
        }

        let header_size = if SOLIDITY_COMP { 8 } else { Self::HEADER_SIZE };

        println!("Data offset: {:?}", data_offset);
        println!("Buf {:?}", &buf.chunk()[..]);

        let mut input_bytes = read_bytes::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_header_el_size,
            val_size,
        )?;
        let real_values = input_bytes.to_vec();
        println!("Real values: {:?}", real_values);
        let mut result = Vec::with_capacity(data_bytes_len);
        println!("Input bytes len: {:?}", input_bytes.len());
        println!("input bytes: {:?}", input_bytes.to_vec());

        for i in 0..data_len {
            let elem_offset = i * align_up::<ALIGN>(T::HEADER_SIZE);

            let value = T::decode::<B, ALIGN, SOLIDITY_COMP>(&mut input_bytes, elem_offset)?;

            result.push(value);
        }

        Ok(result)
    }

    /// Partial decode is used to get the offset and length of the vector without decoding the whole vector.
    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let header_aligned_el_size = align_up::<ALIGN>(4);

        read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset + header_aligned_el_size)

        // if SOLIDITY_COMP {
        //     println!("!!!!!!!!>>>>");
        //     let data_offset = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset)? as usize;
        //     let data_len = read_u32_aligned::<B, ALIGN, true>(buf, data_offset)? as usize;
        //     Ok((aligned_offset, data_len))
        // } else {
        //     let data_len = read_u32_aligned::<B, ALIGN, false>(
        //         buf,
        //         aligned_offset + (header_aligned_el_size * 2),
        //     )? as usize;
        //     Ok((aligned_offset + header_aligned_el_size, data_len))
        // }
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

// fn decode_vec_wasm<
//     B: ByteOrder,
//     T: Default + Sized + Encoder + std::fmt::Debug,
//     const ALIGN: usize,
// >(
//     buf: &(impl Buf + ?Sized),
//     offset: usize,
// ) -> Result<Vec<T>, CodecError> {
//     let aligned_offset = align_up::<ALIGN>(offset);
//     let aligned_header_el_size = align_up::<ALIGN>(4);

//     if buf.remaining() < aligned_offset + aligned_header_el_size {
//         return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
//             expected: aligned_offset + aligned_header_el_size,
//             found: buf.remaining(),
//             msg: "failed to decode vector length".to_string(),
//         }));
//     }

//     let data_len = read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset) as usize;
//     if data_len == 0 {
//         return Ok(Vec::new());
//     }

//     let elem_size = core::mem::size_of::<T>();

//     let input_bytes =
//         read_bytes::<B, ALIGN, false>(buf, aligned_offset + aligned_header_el_size, elem_size)?;

//     decode_elements::<B, T, ALIGN, false>(&input_bytes, data_len)
// }

// fn decode_vec_solidity<
//     B: ByteOrder,
//     T: Default + Sized + Encoder + std::fmt::Debug,
//     const ALIGN: usize,
// >(
//     buf: &(impl Buf + ?Sized),
//     offset: usize,
// ) -> Result<Vec<T>, CodecError> {
//     let aligned_offset = align_up::<ALIGN>(offset);
//     let aligned_header_el_size = align_up::<ALIGN>(4);

//     if buf.remaining() < aligned_offset {
//         return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
//             expected: aligned_offset,
//             found: buf.remaining(),
//             msg: "failed to decode vector offset".to_string(),
//         }));
//     }

//     let data_offset = read_u32_aligned::<B, ALIGN, true>(buf, aligned_offset) as usize;

//     if buf.remaining() < data_offset + aligned_header_el_size {
//         return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
//             expected: data_offset + aligned_header_el_size,
//             found: buf.remaining(),
//             msg: "failed to decode vector length".to_string(),
//         }));
//     }

//     let data_len = read_u32_aligned::<B, ALIGN, true>(buf, data_offset) as usize;
//     if data_len == 0 {
//         return Ok(Vec::new());
//     }

//     let elem_size = core::mem::size_of::<T>();

//     let input_bytes =
//         read_bytes::<B, ALIGN, true>(buf, data_offset + aligned_header_el_size, elem_size)?;

//     decode_elements::<B, T, ALIGN, true>(&input_bytes, data_len)
// }

// fn decode_elements<
//     B: ByteOrder,
//     T: Default + Sized + Encoder + std::fmt::Debug,
//     const ALIGN: usize,
//     const SOLIDITY_COMP: bool,
// >(
//     input_bytes: &[u8],
//     data_len: usize,
// ) -> Result<Vec<T>, CodecError> {
//     let mut result = Vec::with_capacity(data_len);
//     let val_size = core::mem::size_of::<T>();

//     let mut input_bytes = input_bytes;
//     for i in 0..data_len {

//         let elem_offset = i * val_size;

//         let value = T::decode::<B, ALIGN, SOLIDITY_COMP>(&mut input_bytes, elem_offset)?;
//         result.push(value);
//     }

//     Ok(result)
// }
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
    fn test_vec_u32_simple() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buf = BytesMut::new();

        original.encode::<BigEndian, 4, false>(&mut buf, 0).unwrap();
        let mut encoded = buf.freeze();

        let expected_encoded = "000000050000000c000000140000000100000002000000030000000400000005";
        assert_eq!(hex::encode(&encoded), expected_encoded);

        let (data_offset, data_length) =
            read_bytes_header::<BigEndian, 4, false>(&encoded, 4).unwrap();
        println!(
            "MSDFS Data offset: {}, data length: {}",
            data_offset, data_length
        );
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

        let decoded: Vec<u8> =
            Vec::<u8>::decode::<LittleEndian, 4, false>(&mut encoded, 3).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec_le_a2() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buf = BytesMut::new();
        original
            .encode::<LittleEndian, 2, false>(&mut buf, 0)
            .unwrap();
        let mut encoded = buf.freeze();

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
