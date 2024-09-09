extern crate alloc;
use crate::{
    encoder2::{Alignment, Encoder, Endianness},
    evm::{read_bytes, read_bytes_header, write_bytes},
};

use alloc::vec::Vec;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::hash::Hash;
use hashbrown::{HashMap, HashSet};

impl<K: Default + Sized + Encoder<K> + Eq + Hash + Ord, V: Default + Sized + Encoder<V>>
    Encoder<HashMap<K, V>> for HashMap<K, V>
{
    // length + keys (bytes) + values (bytes)
    const HEADER_SIZE: usize = 4 + 8 + 8;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = A::align(offset);

        // Make sure buffer is large enough to store map size
        if buffer.len() < aligned_offset + Self::HEADER_SIZE {
            buffer.resize(aligned_offset + Self::HEADER_SIZE, 0);
        }

        // Write map size
        E::write_u32(
            &mut buffer[aligned_offset..aligned_offset + 4],
            self.len() as u32,
        );

        // Make sure keys & values are sorted
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        // Encode keys
        let mut key_buffer = BytesMut::zeroed(A::SIZE.max(K::HEADER_SIZE) * self.len());
        for (i, (key, _)) in entries.iter().enumerate() {
            let offset = A::SIZE.max(K::HEADER_SIZE) * i;
            key.encode::<A, E>(&mut key_buffer, offset);
        }
        // Write keys
        write_bytes::<A, E>(buffer, aligned_offset + 4, &key_buffer);

        // Encode values
        let mut value_buffer = BytesMut::zeroed(A::SIZE.max(V::HEADER_SIZE) * self.len());
        for (i, (_, value)) in entries.iter().enumerate() {
            let offset = A::SIZE.max(V::HEADER_SIZE) * i;
            value.encode::<A, E>(&mut value_buffer, offset);
        }

        // Write values
        write_bytes::<A, E>(buffer, aligned_offset + 12, &value_buffer);
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        offset: usize,
        result: &mut HashMap<K, V>,
    ) -> (usize, usize) {
        let aligned_offset = A::align(offset);

        if bytes.len() < aligned_offset + Self::HEADER_SIZE {
            return (0, 0);
        }

        let map_len = E::read_u32(&bytes[aligned_offset..aligned_offset + 4]) as usize;

        if map_len == 0 {
            result.clear();
            return (0, 0);
        }

        result.reserve(map_len);

        let (keys_offset, keys_length) = read_bytes_header::<A, E>(bytes, aligned_offset + 4);
        let (_, values_length) = read_bytes_header::<A, E>(bytes, aligned_offset + 12);
        // sum of keys and values are total body length
        (keys_offset, keys_length + values_length)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        offset: usize,
        result: &mut HashMap<K, V>,
    ) {
        let aligned_offset = A::align(offset);
        let map_len = E::read_u32(&bytes[aligned_offset..aligned_offset + 4]) as usize;

        if map_len == 0 {
            result.clear();
            return;
        }

        let key_bytes = read_bytes::<A, E>(bytes, aligned_offset + 4);

        let value_bytes = read_bytes::<A, E>(bytes, aligned_offset + 12);

        let keys = (0..map_len).map(|i| {
            let mut result = Default::default();
            let offset = A::SIZE.max(K::HEADER_SIZE) * i;
            K::decode_body::<A, E>(&key_bytes, offset, &mut result);
            result
        });

        let mut value_bytes = Bytes::from(value_bytes);
        let values = (0..map_len).map(|i| {
            let mut result = Default::default();
            let offset = A::SIZE.max(V::HEADER_SIZE) * i;
            V::decode_body::<A, E>(&mut value_bytes, offset, &mut result);
            result
        });

        *result = keys.zip(values).collect();
    }
}

impl<T: Default + Sized + Encoder<T> + Eq + Hash + Ord> Encoder<HashSet<T>> for HashSet<T> {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;

    fn encode<A: Alignment, E: Endianness>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = A::align(offset);

        if buffer.len() < aligned_offset + 4 {
            buffer.resize(aligned_offset + 4, 0);
        }

        // HashSet size
        E::write_u32(
            &mut buffer[aligned_offset..aligned_offset + 4],
            self.len() as u32,
        );

        if self.is_empty() {
            return;
        }

        // Make sure set is sorted
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort();

        // Encode values
        let mut value_buffer = BytesMut::with_capacity(T::HEADER_SIZE * self.len());
        for (i, obj) in entries.iter().enumerate() {
            obj.encode::<A, E>(&mut value_buffer, T::HEADER_SIZE * i);
        }

        // Write values
        write_bytes::<A, E>(buffer, aligned_offset + 4, &value_buffer);
    }

    fn decode_header<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        field_offset: usize,
        result: &mut HashSet<T>,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);

        if bytes.remaining() < aligned_offset + 4 {
            return (0, 0);
        }

        let count = E::read_u32(&bytes[aligned_offset..aligned_offset + 4]) as usize;

        if count == 0 {
            result.clear();
            return (0, 0);
        }

        result.reserve(count);

        let (data_offset, data_length) = read_bytes_header::<A, E>(bytes, aligned_offset + 4);

        (data_offset, data_length)
    }

    fn decode_body<A: Alignment, E: Endianness>(
        bytes: &Bytes,
        offset: usize,
        result: &mut HashSet<T>,
    ) {
        let aligned_offset = A::align(offset);
        let count = E::read_u32(&bytes[aligned_offset..aligned_offset + 4]) as usize;

        if count == 0 {
            result.clear();
            return;
        }

        let value_bytes = read_bytes::<A, E>(bytes, aligned_offset + 4);

        let elem_size = A::SIZE.max(T::HEADER_SIZE);

        for i in 0..count {
            let mut value = T::default();
            T::decode_body::<A, E>(&value_bytes, elem_size * i, &mut value);
            result.insert(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder2::{Align0, Align4, Align8, BigEndian, LittleEndian};
    use alloc::vec::Vec;
    use bytes::BytesMut;
    use hashbrown::HashMap;

    #[test]
    fn test_map_cur() {
        let mut values = HashMap::new();
        values.insert(100, 20);
        values.insert(3, 5);
        values.insert(1000, 60);
        let mut buffer = BytesMut::new();
        values.encode::<Align0, LittleEndian>(&mut buffer, 0);
        let result = buffer.freeze();

        let encoded_hex = hex::encode(&result);
        println!("Encoded: {}", encoded_hex);

        // Note: The expected encoded string might need to be updated based on the new encoding format
        let expected_encoded = "03000000140000000c000000200000000c0000000300000064000000e803000005000000140000003c000000";
        assert_eq!(encoded_hex, expected_encoded, "Encoding mismatch");

        let bytes = result.clone();
        let mut values2 = Default::default();
        HashMap::decode_body::<Align0, LittleEndian>(&bytes, 0, &mut values2);
        assert_eq!(values, values2);

        let bytes = result.clone();
        let mut values2: HashMap<i32, i32> = HashMap::new();
        let (offset, length) =
            HashMap::decode_header::<Align0, LittleEndian>(&bytes, 0, &mut values2);

        assert_eq!(offset, 20);
        assert_eq!(length, 24);
    }

    #[test]
    fn test_nested_map() {
        let mut values = HashMap::new();
        values.insert(100, HashMap::from([(1, 2), (3, 4)]));
        values.insert(3, HashMap::new());
        values.insert(1000, HashMap::from([(7, 8), (9, 4)]));

        let mut buffer = BytesMut::new();

        values.encode::<Align0, LittleEndian>(&mut buffer, 0);
        let result = buffer.freeze();
        println!("{}", hex::encode(&result));

        // Note: The expected encoded string might need to be updated based on the new encoding format
        let expected_encoded = "03000000140000000c000000200000005c0000000300000064000000e8030000000000003c000000000000003c00000000000000020000003c000000080000004400000008000000020000004c0000000800000054000000080000000100000003000000020000000400000007000000090000000800000004000000";

        assert_eq!(hex::encode(&result), expected_encoded, "Encoding mismatch");

        let mut bytes = result.clone();
        let mut values2 = Default::default();
        HashMap::decode_body::<Align0, LittleEndian>(&mut bytes, 0, &mut values2);
        assert_eq!(values, values2);
    }

    #[test]
    fn test_vector_of_maps() {
        let mut values = Vec::new();
        values.push(HashMap::from([(1, 2), (3, 4)]));
        values.push(HashMap::new());
        values.push(HashMap::from([(7, 8), (9, 4)]));
        let mut buffer = BytesMut::new();
        values.encode::<Align4, LittleEndian>(&mut buffer, 0);

        let result = buffer.freeze();
        println!("{}", hex::encode(&result));
        let expected_encoded = "030000000c0000005c000000020000003c000000080000004400000008000000000000004c000000000000004c00000000000000020000004c0000000800000054000000080000000100000003000000020000000400000007000000090000000800000004000000";

        assert_eq!(hex::encode(&result), expected_encoded, "Encoding mismatch");
        let mut bytes = result.clone();
        let mut values2 = Default::default();
        Vec::decode_body::<Align4, LittleEndian>(&mut bytes, 0, &mut values2);
        assert_eq!(values, values2);
    }

    #[test]
    fn test_map_of_vectors() {
        let mut values = HashMap::new();
        values.insert(vec![0, 1, 2], vec![3, 4, 5]);
        values.insert(vec![3, 1, 2], vec![3, 4, 5]);
        values.insert(vec![0, 1, 6], vec![3, 4, 5]);
        let mut buffer = BytesMut::new();
        values.encode::<Align0, LittleEndian>(&mut buffer, 0);
        let result = buffer.freeze();

        // Note: The expected encoded string might need to be updated based on the new encoding format
        let expected_encoded = "0300000014000000480000005c0000004800000003000000240000000c00000003000000300000000c000000030000003c0000000c00000000000000010000000200000000000000010000000600000003000000010000000200000003000000240000000c00000003000000300000000c000000030000003c0000000c000000030000000400000005000000030000000400000005000000030000000400000005000000";
        assert_eq!(hex::encode(&result), expected_encoded, "Encoding mismatch");

        let mut values2 = HashMap::new();
        HashMap::<Vec<i32>, Vec<i32>>::decode_body::<Align0, LittleEndian>(
            &result,
            0,
            &mut values2,
        );
        assert_eq!(values, values2);
    }
}
