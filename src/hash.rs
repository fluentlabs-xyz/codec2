extern crate alloc;
use alloc::vec::Vec;
use core::hash::Hash;

use byteorder::ByteOrder;

use bytes::{Buf, BytesMut};
use hashbrown::{HashMap, HashSet};

use crate::{
    bytes::{read_bytes_header, write_bytes},
    encoder::{align_up, read_u32_aligned, write_u32_aligned, Encoder},
};

use crate::error::{CodecError, DecodingError};

/// Example of encoding a nested HashMap:
///
/// ```rust
/// use hashbrown::HashMap;
///
/// let mut values = HashMap::new();
/// values.insert(100, HashMap::from([(1, 2), (3, 4)]));
/// values.insert(3, HashMap::new());
/// values.insert(1000, HashMap::from([(7, 8), (9, 4)]));
/// ```
///
/// Encoded data (in hexadecimal):
/// ```text
/// 03000000140000000c000000200000005c000000 // Header of outer HashMap
/// 0300000064000000e8030000                 // Keys of outer HashMap
/// 00000000 3c000000 00000000 3c000000 00000000 // Empty inner HashMap (key 3)
/// 02000000 3c000000 08000000 44000000 08000000 // Header of inner HashMap (key 100)
/// 01000000 03000000 02000000 04000000         // Data of inner HashMap (key 100)
/// 02000000 4c000000 08000000 54000000 08000000 // Header of inner HashMap (key 1000)
/// 07000000 09000000 08000000 04000000         // Data of inner HashMap (key 1000)
/// ```
///
/// Detailed explanation:
/// ```text
/// // Outer HashMap header
/// 03000000 - Number of elements in outer HashMap (3)
/// 14000000 - Offset to keys of outer HashMap (20 bytes)
/// 0c000000 - Length of keys data in outer HashMap (12 bytes)
/// 20000000 - Offset to values of outer HashMap (32 bytes)
/// 5c000000 - Length of values data in outer HashMap (92 bytes)
///
/// // Outer HashMap keys (sorted)
/// 03000000 - Key 1 (3)
/// 64000000 - Key 2 (100)
/// e8030000 - Key 3 (1000)
///
/// // Outer HashMap values (inner HashMaps)
///
/// // Value for key 3 (empty HashMap)
/// 00000000 - Number of elements (0)
/// 3c000000 - Offset to keys (unused)
/// 00000000 - Length of keys data (0)
/// 3c000000 - Offset to values (unused)
/// 00000000 - Length of values data (0)
///
/// // Value for key 100 (HashMap with 2 elements)
/// 02000000 - Number of elements (2)
/// 3c000000 - Offset to keys (60 bytes from start of this inner HashMap)
/// 08000000 - Length of keys data (8 bytes)
/// 44000000 - Offset to values (68 bytes from start of this inner HashMap)
/// 08000000 - Length of values data (8 bytes)
/// 01000000 - Inner key 1 (1)
/// 03000000 - Inner key 2 (3)
/// 02000000 - Value for inner key 1 (2)
/// 04000000 - Value for inner key 2 (4)
///
/// // Value for key 1000 (HashMap with 2 elements)
/// 02000000 - Number of elements (2)
/// 4c000000 - Offset to keys (76 bytes from start of this inner HashMap)
/// 08000000 - Length of keys data (8 bytes)
/// 54000000 - Offset to values (84 bytes from start of this inner HashMap)
/// 08000000 - Length of values data (8 bytes)
/// 07000000 - Inner key 1 (7)
/// 09000000 - Inner key 2 (9)
/// 08000000 - Value for inner key 1 (8)
/// 04000000 - Value for inner key 2 (4)
/// ```
///
/// Notes:
/// - All integers are stored in little-endian format.
/// - Keys in both outer and inner HashMaps are sorted.
/// - Empty HashMaps (like the one for key 3) still have a full header, but with zero lengths.
/// - Offsets in inner HashMaps are relative to the start of that inner HashMap's data.
impl<K, V> Encoder for HashMap<K, V>
where
    K: Default + Sized + Encoder + Eq + Hash + Ord,
    V: Default + Sized + Encoder,
{
    const HEADER_SIZE: usize = 4 + 8 + 8; // length + keys_header + values_header

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_el_size = align_up::<ALIGN>(4);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        // Ensure buf is large enough for the header
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }

        // Write map size
        write_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset, self.len() as u32);

        // Make sure keys & values are sorted
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        // Encode and write keys (we can't write values, as we need to store all keys first(including nested))

        let mut key_buf = BytesMut::zeroed(align_up::<ALIGN>(K::HEADER_SIZE) * self.len());

        for (i, (key, _)) in entries.iter().enumerate() {
            let key_offset = align_up::<ALIGN>(K::HEADER_SIZE) * i;
            key.encode::<B, ALIGN, SOLIDITY_COMP>(&mut key_buf, key_offset)
                .unwrap();
        }

        write_bytes::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_header_el_size,
            &key_buf,
        );

        let mut value_buf = BytesMut::zeroed(align_up::<ALIGN>(V::HEADER_SIZE) * self.len());
        // Encode and write values
        for (i, (_, value)) in entries.iter().enumerate() {
            let value_offset = align_up::<ALIGN>(V::HEADER_SIZE) * i;
            value
                .encode::<B, ALIGN, SOLIDITY_COMP>(&mut value_buf, value_offset)
                .unwrap();
        }

        write_bytes::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_header_el_size * 3,
            &value_buf,
        );

        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_size,
                found: buf.remaining(),
                msg: "Not enough data to decode HashMap header".to_string(),
            }));
        }

        let length = read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset) as usize;

        let (keys_offset, keys_length) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + align_up::<ALIGN>(4),
        )
        .unwrap();
        let (values_offset, values_length) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + align_up::<ALIGN>(12),
        )
        .unwrap();

        let mut result = HashMap::with_capacity(length);

        let key_bytes = &buf.chunk()[keys_offset..keys_offset + keys_length];
        let value_bytes = &buf.chunk()[values_offset..values_offset + values_length];

        let keys = (0..length).map(|i| {
            let key_offset = align_up::<ALIGN>(K::HEADER_SIZE) * i;
            K::decode::<B, ALIGN, SOLIDITY_COMP>(&key_bytes, key_offset).unwrap_or_default()
        });

        let values = (0..length).map(|i| {
            let value_offset = align_up::<ALIGN>(V::HEADER_SIZE) * i;
            V::decode::<B, ALIGN, SOLIDITY_COMP>(&value_bytes, value_offset).unwrap_or_default()
        });

        result = keys.zip(values).collect();

        if result.len() != length {
            return Err(CodecError::Decoding(DecodingError::InvalidData(format!(
                "Expected {} elements, but decoded {}",
                length,
                result.len()
            ))));
        }

        Ok(result)
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_size,
                found: buf.remaining(),
                msg: "Not enough data to decode HashMap header".to_string(),
            }));
        }

        let (keys_offset, keys_length) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + align_up::<ALIGN>(4),
        )
        .unwrap();
        let (_values_offset, values_length) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + align_up::<ALIGN>(12),
        )
        .unwrap();

        Ok((keys_offset, keys_length + values_length))
    }
}

impl<T> Encoder for HashSet<T>
where
    T: Default + Sized + Encoder + Eq + Hash + Ord,
{
    const HEADER_SIZE: usize = 4 + 8; // length + data_header

    fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        &self,
        buf: &mut BytesMut,
        offset: usize,
    ) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_el_size = align_up::<ALIGN>(4);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        // Ensure buf is large enough for the header
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }

        // Write set size
        write_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset, self.len() as u32);

        // Make sure set is sorted
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort();

        // Encode values
        let mut value_buf = BytesMut::zeroed(align_up::<ALIGN>(T::HEADER_SIZE) * self.len());
        for (i, value) in entries.iter().enumerate() {
            let value_offset = align_up::<ALIGN>(T::HEADER_SIZE) * i;
            value.encode::<B, ALIGN, SOLIDITY_COMP>(&mut value_buf, value_offset)?;
        }

        // Write values
        write_bytes::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + aligned_header_el_size,
            &value_buf,
        );

        Ok(())
    }

    fn decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<Self, CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_size,
                found: buf.remaining(),
                msg: "Not enough data to decode HashSet header".to_string(),
            }));
        }

        let length = read_u32_aligned::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset) as usize;

        let (data_offset, data_length) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + align_up::<ALIGN>(4),
        )?;

        let mut result = HashSet::with_capacity(length);

        let value_bytes = &buf.chunk()[data_offset..data_offset + data_length];

        for i in 0..length {
            let value_offset = align_up::<ALIGN>(T::HEADER_SIZE) * i;
            let value = T::decode::<B, ALIGN, SOLIDITY_COMP>(&value_bytes, value_offset)?;
            result.insert(value);
        }

        if result.len() != length {
            return Err(CodecError::Decoding(DecodingError::InvalidData(format!(
                "Expected {} elements, but decoded {}",
                length,
                result.len()
            ))));
        }

        Ok(result)
    }

    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);

        if buf.remaining() < aligned_offset + aligned_header_size {
            return Err(CodecError::Decoding(DecodingError::BufferTooSmall {
                expected: aligned_offset + aligned_header_size,
                found: buf.remaining(),
                msg: "Not enough data to decode HashSet header".to_string(),
            }));
        }

        let (data_offset, data_length) = read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(
            buf,
            aligned_offset + align_up::<ALIGN>(4),
        )?;

        Ok((data_offset, data_length))
    }
}
// impl<T: Default + Sized + Encoder<T> + Eq + Hash + Ord> Encoder<HashSet<T>> for HashSet<T> {
//     // length (4) + keys (8) (bytes)
//     const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;

//     fn encode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(&self, buf: &mut BytesMut, offset: usize) {
//         let aligned_offset = A::align(offset);

//         if buf.len() < aligned_offset + 4 {
//             buf.resize(aligned_offset + 4, 0);
//         }

//         // HashSet size
//         E::write::<u32>(
//             &mut buf[aligned_offset..aligned_offset + 4],
//             self.len() as u32,
//         );

//         // Make sure set is sorted
//         let mut entries: Vec<_> = self.iter().collect();
//         entries.sort();

//         // Encode values
//         let mut value_buffer = BytesMut::zeroed(A::SIZE.max(T::HEADER_SIZE) * self.len());
//         for (i, obj) in entries.iter().enumerate() {
//             let offset = A::SIZE.max(T::HEADER_SIZE) * i;
//             obj.encode::<A, E>(&mut value_buffer, offset);
//         }

//         // Write values
//         write_bytes::<A, E>(buf, aligned_offset + 4, &value_buffer);
//     }

//     fn decode_header<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
//         bytes: &Bytes,
//         field_offset: usize,
//         result: &mut HashSet<T>,
//     ) -> (usize, usize) {
//         let aligned_offset = A::align(field_offset);

//         if bytes.remaining() < aligned_offset + 4 {
//             return (0, 0);
//         }

//         let count = E::read::<u32>(&bytes[aligned_offset..aligned_offset + 4]) as usize;

//         if count == 0 {
//             result.clear();
//             return (0, 0);
//         }

//         result.reserve(count);

//         let (data_offset, data_length) = read_bytes_header::<A, E>(bytes, aligned_offset + 4);

//         (data_offset, data_length)
//     }

//     fn decode_body<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
//         bytes: &Bytes,
//         offset: usize,
//         result: &mut HashSet<T>,
//     ) {
//         let aligned_offset = A::align(offset);
//         let count = E::read::<u32>(&bytes[aligned_offset..aligned_offset + 4]) as usize;

//         if count == 0 {
//             result.clear();
//             return;
//         }

//         let value_bytes = read_bytes::<A, E>(bytes, aligned_offset + 4);

//         let elem_size = A::SIZE.max(T::HEADER_SIZE);

//         for i in 0..count {
//             let mut value = T::default();
//             T::decode_body::<A, E>(&value_bytes, elem_size * i, &mut value);
//             result.insert(value);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use byteorder::LittleEndian;
    use bytes::BytesMut;
    use hashbrown::HashMap;

    use super::*;

    #[test]
    fn test_nested_map() {
        let mut values = HashMap::new();
        values.insert(100, HashMap::from([(1, 2), (3, 4)]));
        values.insert(3, HashMap::new());
        values.insert(1000, HashMap::from([(7, 8), (9, 4)]));
        let expected_encoded = "03000000140000000c000000200000005c0000000300000064000000e8030000000000003c000000000000003c00000000000000020000003c000000080000004400000008000000020000004c0000000800000054000000080000000100000003000000020000000400000007000000090000000800000004000000";

        let mut buf = BytesMut::new();
        values
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();
        let encoded = buf.freeze();

        assert_eq!(hex::encode(&encoded), expected_encoded, "Encoding mismatch");

        let decoded =
            HashMap::<i32, HashMap<i32, i32>>::decode::<LittleEndian, 4, false>(&encoded, 0)
                .unwrap();
        assert_eq!(values, decoded);

        let header = HashMap::<i32, HashMap<i32, i32>>::partial_decode::<LittleEndian, 4, false>(
            &encoded, 0,
        )
        .unwrap();

        assert_eq!(header, (20, 104));
        println!("Header: {:?}", header);
    }

    #[test]
    fn test_simple_map() {
        let mut values = HashMap::new();
        values.insert(100, 20);
        values.insert(3, 5);
        values.insert(1000, 60);
        let mut buf = BytesMut::new();
        values
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();
        let result = buf.freeze();

        let encoded_hex = hex::encode(&result);
        println!("Encoded: {}", encoded_hex);

        let decoded = HashMap::<i32, i32>::decode::<LittleEndian, 4, false>(&result, 0).unwrap();
        assert_eq!(values, decoded);
    }

    #[test]
    fn test_vector_of_maps() {
        let values = vec![
            HashMap::from([(1, 2), (3, 4)]),
            HashMap::new(),
            HashMap::from([(7, 8), (9, 4)]),
        ];

        let mut buf = BytesMut::new();
        values
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();

        let result = buf.freeze();
        println!("{}", hex::encode(&result));
        let expected_encoded = "030000000c0000005c000000020000003c000000080000004400000008000000000000004c000000000000004c00000000000000020000004c0000000800000054000000080000000100000003000000020000000400000007000000090000000800000004000000";

        assert_eq!(hex::encode(&result), expected_encoded, "Encoding mismatch");
        let bytes = result.clone();
        let values2 = Vec::decode::<LittleEndian, 4, false>(&bytes, 0).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn test_map_of_vectors() {
        let mut values = HashMap::new();
        values.insert(vec![0, 1, 2], vec![3, 4, 5]);
        values.insert(vec![3, 1, 2], vec![3, 4, 5]);
        values.insert(vec![0, 1, 6], vec![3, 4, 5]);
        let mut buf = BytesMut::new();
        values
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();
        let result = buf.freeze();

        // Note: The expected encoded string might need to be updated based on the new encoding format
        let expected_encoded = "0300000014000000480000005c0000004800000003000000240000000c00000003000000300000000c000000030000003c0000000c00000000000000010000000200000000000000010000000600000003000000010000000200000003000000240000000c00000003000000300000000c000000030000003c0000000c000000030000000400000005000000030000000400000005000000030000000400000005000000";
        assert_eq!(hex::encode(&result), expected_encoded, "Encoding mismatch");

        let values2 =
            HashMap::<Vec<i32>, Vec<i32>>::decode::<LittleEndian, 4, false>(&result, 0).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn test_set() {
        let values = HashSet::from([1, 2, 3]);
        let mut buf = BytesMut::new();
        values
            .encode::<LittleEndian, 4, false>(&mut buf, 0)
            .unwrap();
        let result = buf.freeze();

        println!("{}", hex::encode(&result));
        let expected_encoded = "030000000c0000000c000000010000000200000003000000";
        assert_eq!(hex::encode(&result), expected_encoded, "Encoding mismatch");

        let values2 = HashSet::<i32>::decode::<LittleEndian, 4, false>(&result, 0).unwrap();
        assert_eq!(values, values2);
    }

    #[test]
    fn test_set_is_sorted() {
        let values1 = HashSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let mut buffer1 = BytesMut::new();
        values1
            .encode::<LittleEndian, 4, false>(&mut buffer1, 0)
            .unwrap();
        let result1 = buffer1.freeze();

        let values2 = HashSet::from([8, 3, 2, 4, 5, 9, 7, 1, 6]);
        let mut buffer2 = BytesMut::new();
        values2
            .encode::<LittleEndian, 4, false>(&mut buffer2, 0)
            .unwrap();
        let result2 = buffer2.freeze();

        assert_eq!(result1, result2);
    }
}
