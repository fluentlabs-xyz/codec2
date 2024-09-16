extern crate alloc;
use crate::{
    encoder::{Alignment, Encoder, Endian},
    evm::{read_bytes, write_bytes},
};
use alloc::vec::Vec;
use bytes::{Buf, Bytes, BytesMut};

pub trait SolidityVecEncoding<T: Default + Sized + Encoder<T>> {
    fn encode_abi<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, offset: usize);
    fn decode_abi_header<A: Alignment, E: Endian>(
        bytes: &Bytes,
        field_offset: usize,
    ) -> (usize, usize);
    fn decode_abi_body<A: Alignment, E: Endian>(bytes: &Bytes, offset: usize, result: &mut Vec<T>);
}

// Пример вложенного массива
// [[1, 2], [3, 4, 5]]

// 0000000000000000000000000000000000000000000000000000000000000020  // (1) Смещение до начала данных внешнего массива (32)
// ---------------------------------------------------------------------
// 0000000000000000000000000000000000000000000000000000000000000002  // (2) Длина внешнего массива (2)
// 0000000000000000000000000000000000000000000000000000000000000040  // (3) Смещение до первого внутреннего массива (64)
// 00000000000000000000000000000000000000000000000000000000000000a0  // (4) Смещение до второго внутреннего массива (160)
// ---------------------------------------------------------------------
// 0000000000000000000000000000000000000000000000000000000000000002  // (5) Длина первого внутреннего массива (2)
// ---------------------------------------------------------------------
// 0000000000000000000000000000000000000000000000000000000000000001  // (6) Элемент 1 первого внутреннего массива
// 0000000000000000000000000000000000000000000000000000000000000002  // (7) Элемент 2 первого внутреннего массива
// ---------------------------------------------------------------------
// 0000000000000000000000000000000000000000000000000000000000000003  // (8) Длина второго внутреннего массива (3)
// ---------------------------------------------------------------------
// 0000000000000000000000000000000000000000000000000000000000000003  // (9) Элемент 1 второго внутреннего массива
// 0000000000000000000000000000000000000000000000000000000000000004  // (10) Элемент 2 второго внутреннего массива
// 0000000000000000000000000000000000000000000000000000000000000005  // (11) Элемент 3 второго внутреннего массива
impl<T: Default + Sized + Encoder<T>> SolidityVecEncoding<T> for Vec<T> {
    fn encode_abi<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = A::align(offset);

        let elem_size = A::align(32);

        if buffer.len() < aligned_offset + elem_size {
            buffer.resize(aligned_offset + elem_size, 0);
        }

        // Data offset
        let data_offset = aligned_offset + 32;
        E::write::<u32>(
            &mut buffer[aligned_offset..aligned_offset + elem_size],
            data_offset as u32,
        );

        // encode values
        // reserve space for headers
        let mut value_encoder = BytesMut::zeroed(A::SIZE.max(T::HEADER_SIZE) * self.len());

        for (index, obj) in self.iter().enumerate() {
            let elem_offset = A::SIZE.max(T::HEADER_SIZE) * index;
            obj.encode::<A, E>(&mut value_encoder, elem_offset);
        }

        write_bytes::<A, E>(buffer, aligned_offset + 4, &value_encoder.freeze());
    }

    fn decode_abi_header<A: Alignment, E: Endian>(
        bytes: &Bytes,
        field_offset: usize,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);

        // Read offset to data
        let data_offset = E::read::<u32>(&bytes[aligned_offset + 28..aligned_offset + 32]) as usize;

        // Read length of vector
        let length = E::read::<u32>(&bytes[data_offset + 28..data_offset + 32]) as usize;

        (data_offset + 32, length)
    }

    fn decode_abi_body<A: Alignment, E: Endian>(bytes: &Bytes, offset: usize, result: &mut Vec<T>) {
        let (data_offset, length) = Self::decode_abi_header::<A, E>(bytes, offset);

        result.clear();
        result.reserve(length);

        let mut current_offset = data_offset;
        for _ in 0..length {
            let mut item = T::default();
            T::decode_body::<A, E>(bytes, current_offset, &mut item);
            result.push(item);
            current_offset += T::HEADER_SIZE;
        }
    }
}

///
/// We encode dynamic arrays as following:
/// - header
/// - + length - number of elements inside vector
/// - + offset - offset inside structure
/// - + size - number of encoded bytes
/// - body
/// - + raw bytes of the vector
///
/// We don't encode empty vectors, instead we store 0 as length,
/// it helps to reduce empty vector size from 12 to 4 bytes.
impl<T: Default + Sized + Encoder<T>> Encoder<Vec<T>> for Vec<T> {
    const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;

    fn encode<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, offset: usize) {
        let aligned_offset = A::align(offset);

        let elem_size = A::align(4);

        if buffer.len() < aligned_offset + elem_size {
            buffer.resize(aligned_offset + elem_size, 0);
        }

        // Vector size
        E::write::<u32>(
            &mut buffer[aligned_offset..aligned_offset + elem_size],
            self.len() as u32,
        );

        // encode values
        // reserve space for headers
        let mut value_encoder = BytesMut::zeroed(A::SIZE.max(T::HEADER_SIZE) * self.len());

        for (index, obj) in self.iter().enumerate() {
            let elem_offset = A::SIZE.max(T::HEADER_SIZE) * index;
            obj.encode::<A, E>(&mut value_encoder, elem_offset);
        }

        write_bytes::<A, E>(buffer, aligned_offset + 4, &value_encoder.freeze());
    }

    fn decode_header<A: Alignment, E: Endian>(
        bytes: &bytes::Bytes,
        field_offset: usize,
        result: &mut Vec<T>,
    ) -> (usize, usize) {
        let aligned_offset = A::align(field_offset);
        let elem_size = A::align(4);

        // TODO: d1r1 maybe we should return an error here?
        if bytes.remaining() < aligned_offset + elem_size {
            return (0, 0);
        }

        // Vector size
        let count = E::read::<u32>(&bytes[aligned_offset..aligned_offset + elem_size]) as usize;

        // If vector is empty, we don't need to decode anything
        if count == 0 {
            result.clear();
            return (0, 0);
        }

        // Get data offset and length
        let data_offset =
            E::read::<u32>(&bytes[aligned_offset + elem_size..aligned_offset + elem_size * 2])
                as usize;
        let data_length =
            E::read::<u32>(&bytes[aligned_offset + elem_size * 2..aligned_offset + elem_size * 3])
                as usize;

        result.reserve(data_length);

        (data_offset, data_length)
    }

    fn decode_body<A: Alignment, E: Endian>(bytes: &Bytes, offset: usize, result: &mut Vec<T>) {
        let aligned_offset = A::align(offset);
        let elem_size = A::align(4);
        let data_len = E::read::<u32>(&bytes[aligned_offset..aligned_offset + elem_size]) as usize;

        if data_len == 0 {
            result.clear();
            return;
        }

        let input_bytes = read_bytes::<A, E>(bytes, aligned_offset + elem_size);

        let elem_size = A::SIZE.max(T::HEADER_SIZE);
        *result = (0..data_len)
            .map(|i| {
                let mut value = T::default();
                let elem_offset = elem_size * i;
                T::decode_body::<A, E>(&input_bytes, elem_offset, &mut value);
                value
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{Align0, Align4, Align8, BigEndian, Encoder, LittleEndian};

    #[test]
    fn test_empty_vec_u32() {
        let original: Vec<u32> = Vec::new();
        let mut buffer = BytesMut::new();

        original.encode::<Align4, LittleEndian>(&mut buffer, 0);
        let encoded = buffer.freeze();
        let expected = hex::decode("000000000c00000000000000").expect("Failed to decode hex");
        assert_eq!(encoded, Bytes::from(expected));

        let mut decoded: Vec<u32> = Vec::new();
        Vec::<u32>::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32() {
        let original: Vec<u32> = vec![1, 2, 3, 4];
        let mut buffer = BytesMut::new();

        original.encode::<Align4, BigEndian>(&mut buffer, 0);
        let encoded = buffer.freeze();

        let mut decoded: Vec<u32> = Vec::new();
        Vec::<u32>::decode_body::<Align4, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vec_u32_with_offset() {
        let original: Vec<u32> = vec![1, 2, 3, 4, 5];
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        original.encode::<Align4, LittleEndian>(&mut buffer, 3);
        let encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));

        let mut decoded: Vec<u32> = Vec::new();
        Vec::<u32>::decode_body::<Align4, LittleEndian>(&encoded, 3, &mut decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_vec_u8_with_offset() {
        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&[0xFF, 0xFF, 0xFF]); // Add some initial data

        original.encode::<Align4, LittleEndian>(&mut buffer, 3);
        let encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));

        let mut decoded: Vec<u8> = Vec::new();
        Vec::<u8>::decode_body::<Align4, LittleEndian>(&encoded, 3, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_nested_vec() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buffer = BytesMut::new();
        original.encode::<Align0, LittleEndian>(&mut buffer, 0);
        let encoded = buffer.freeze();
        println!("{:?}", hex::encode(&encoded));
        let expected_encoded = "020000000c00000022000000020000001800000004000000030000001c0000000600000003000400050006000700";

        assert_eq!(hex::encode(&encoded), expected_encoded);

        let mut decoded: Vec<Vec<u16>> = Vec::new();
        Vec::<Vec<u16>>::decode_body::<Align0, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_nested_vec_a4_le() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buffer = BytesMut::new();
        original.encode::<Align4, LittleEndian>(&mut buffer, 0);
        let encoded = buffer.freeze();
        let mut decoded: Vec<Vec<u16>> = Vec::new();
        Vec::<Vec<u16>>::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }
    #[test]
    fn test_nested_vec_a4_be() {
        let original: Vec<Vec<u16>> = vec![vec![3, 4], vec![5, 6, 7]];

        let mut buffer = BytesMut::new();
        original.encode::<Align4, BigEndian>(&mut buffer, 0);
        let encoded = buffer.freeze();

        let mut decoded: Vec<Vec<u16>> = Vec::new();
        Vec::<Vec<u16>>::decode_body::<Align4, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_large_vec() {
        let original: Vec<u64> = (0..1000).collect();
        let mut buffer = BytesMut::new();

        original.encode::<Align8, BigEndian>(&mut buffer, 0);
        let encoded = buffer.freeze();

        let mut decoded: Vec<u64> = Vec::new();
        Vec::<u64>::decode_body::<Align8, BigEndian>(&encoded, 0, &mut decoded);

        assert_eq!(original, decoded);
    }
}
