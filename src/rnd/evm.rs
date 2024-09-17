 use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
 use bytes::{Buf, BufMut, BytesMut};

 // pub fn write_bytes_abi<A: Alignment, E: Endian>(
 //     buffer: &mut BytesMut,
 //     offset: usize,
 //     bytes: &[u8],
 // ) -> usize {
 //     // length
 //     // data
 //     todo!()
 // }

 // u32 (4 bytes)
 const HEADER_ELEMENT_SIZE: usize = 4;
 const HEADER_ELEMENT_COUNT: usize = 2;

 // Is it possible to increase the size of the BufMut buffer?
 //
 pub fn write_bytes<B: ByteOrderExt, const ALIGN: usize>(
     buf: &mut impl BufMut,
     offset: usize,
     data_offset: usize, // offset of the data section (sometimes we need to place the data at a specific offset)
     data: &[u8],
 ) -> Result<(), EncoderError> {
     let aligned_offset = align_offset::<ALIGN>(offset);
     let aligned_data_offset = align_offset::<ALIGN>(data_offset);

     // Check if the buffer has enough space for everything
     let total_required_size = aligned_offset + aligned_data_offset + data.len();
     if buf.remaining_mut() < total_required_size {
         return Err(EncoderError::BufferTooSmall {
             required: total_required_size,
             available: buf.remaining_mut(),
             msg: "Buffer too small to store all data".to_string(),
         });
     }

     // Write header and move mut pointer to the start of the data section
     write_header::<B, ALIGN>(
         buf,
         aligned_offset,
         aligned_data_offset as u32,
         data.len() as u32,
     )?;

     // Somehow I need to check if the buffer has enough space to advance for aligned_offset + aligned_data_offset
     // The challenge is that the buffer is maybe not initialized yet
     // I need to check if the buffer has enough space to advance for aligned_offset + aligned_data_offset

     // Move the buffer to the start of the data section
     unsafe {
         buf.advance_mut(aligned_offset + aligned_data_offset);
     };

     // Write data
     buf.put_slice(data);

     Ok(())
 }

 fn write_header<B: ByteOrderExt, const ALIGN: usize>(
     buf: &mut impl BufMut,
     offset: usize,
     data_offset: u32,
     data_length: u32,
 ) -> Result<(), EncoderError> {
     let write_position = if B::is_big_endian() {
         WritePosition::End
     } else {
         WritePosition::Start
     };

     write_u32_aligned::<B, ALIGN>(buf, offset, data_offset, &write_position)?;
     write_u32_aligned::<B, ALIGN>(buf, offset, data_length, &write_position)?;

     Ok(())
 }

 fn write_u32_aligned<B: ByteOrderExt, const ALIGN: usize>(
     buf: &mut impl BufMut,
     offset: usize,
     value: u32,
     write_position: &WritePosition,
 ) -> Result<(), EncoderError> {
     let elem_size = core::mem::size_of::<u32>();
     let mut temp = vec![0u8; 4];
     B::write_u32(&mut temp, value);

     unsafe { write_slice_aligned::<ALIGN>(buf, offset, &temp, write_position) }
 }

  // Append data
 // buf.put_slice(bytes);

  header_size
 // header_size

 pub fn read_bytes_header<B: ByteOrderExt, const ALIGN: usize>(
     buf: &mut impl Buf,
     offset: usize,
 ) -> Result<(usize, usize), EncoderError> {
     println!("op read_bytes_header");
     let aligned_offset = align_offset::<ALIGN>(offset);
     let elem_size = align_offset::<ALIGN>(core::mem::size_of::<u32>());
     let header_size = elem_size * HEADER_ELEMENT_COUNT;

     if buf.remaining() < aligned_offset + header_size {
         return Err(EncoderError::BufferTooSmall {
             required: aligned_offset + header_size,
             available: buf.remaining(),
             msg: "Buffer too small to read header".to_string(),
         });
     }

     if B::is_big_endian() {
         let offset = B::read_u32(&buf.chunk()[elem_size - 4..elem_size]) as usize;
         let length = B::read_u32(&buf.chunk()[elem_size * 2 - 4..elem_size * 2]) as usize;
         Ok((offset, length))
     } else {
         let offset = B::read_u32(&buf.chunk()[..elem_size]) as usize;
         let length = B::read_u32(&buf.chunk()[elem_size..elem_size * 2]) as usize;
         Ok((offset, length))
     }
 }

 pub fn read_bytes<B: ByteOrderExt, const ALIGN: usize>(
     buf: &mut impl Buf,
     offset: usize,
 ) -> Result<Vec<u8>, EncoderError> {
     let (data_offset, data_length) = read_bytes_header::<B, ALIGN>(buf, offset)?;

     println!("data_offset: {}", data_offset);
     println!("data_length: {}", data_length);

     if buf.remaining() < data_offset + data_length as usize {
         return Err(EncoderError::BufferTooSmall {
             required: data_offset + data_length as usize,
             available: buf.remaining(),
             msg: "Buffer too small to read data".to_string(),
         });
     }

     println!(">>>data: {:?}", buf.chunk()[..].to_vec());
     let data = buf.chunk()[data_offset..].to_vec();
     println!(">>data: {:?}", data);

     Ok(data)
 }

 fn read_u32_aligned<B: ByteOrderExt, const ALIGN: usize>(
     buf: &mut impl Buf,
 ) -> Result<u32, EncoderError> {
     let elem_size = align_offset::<ALIGN>(core::mem::size_of::<u32>());

     if buf.remaining() < elem_size {
         return Err(EncoderError::BufferTooSmall {
             required: elem_size,
             available: buf.remaining(),
             msg: "Buffer too small to read u32".to_string(),
         });
     }

     let mut temp = vec![0u8; elem_size];
     buf.copy_to_slice(&mut temp);
     println!("temp: {:?}", temp);

     Ok(B::read_u32(&temp))
 }

 pub fn read_bytes2<B: ByteOrderExt, const ALIGN: usize>(
     bytes: &mut impl Buf,
     offset: usize,
 ) -> Result<bytes::Bytes, EncoderError> {
     let aligned_offset = align_offset::<ALIGN>(offset);
     let elem_size = align_offset::<ALIGN>(4);
     if bytes.remaining() < elem_size * 2 {
         return Err(EncoderError::NotEnoughData);
     }

     let offset = B::read_u32(&slice[..elem_size]) as usize;
     let length = B::read_u32(&slice[elem_size..elem_size * 2]) as usize;
     bytes.slice(offset..offset + length)
 }

 impl Encoder for Bytes {
     const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;
     const DATA_SIZE: usize = 0; // dynamic

     fn encode<B: ByteOrderExt, const ALIGN: usize>(
         &self,
         buf: &mut impl BufMut,
         offset: usize,
     ) -> Result<(), EncoderError> {
         let aligned_offset = align_offset::<ALIGN>(offset);
         write_bytes::<B, ALIGN>(buf, aligned_offset, Self::HEADER_SIZE, &self.0)
     }

     fn decode<B: ByteOrderExt, const ALIGN: usize>(
         buf: &mut impl bytes::Buf,
         offset: usize,
     ) -> Result<Self, EncoderError> {
         let aligned_offset = align_offset::<ALIGN>(offset);
         let (data_offset, data_length) = read_bytes_header::<B, ALIGN>(buf, aligned_offset)?;

         let total_offset = aligned_offset + data_offset;
         let data = read_bytes::<B, ALIGN>(buf, total_offset)?;

         Ok(Bytes::from(data))
     }

     fn partial_decode<B: ByteOrderExt, const ALIGN: usize>(
         buf: &mut impl bytes::Buf,
         offset: usize,
     ) -> Result<(usize, usize), EncoderError> {
         let aligned_header_offset = align_offset::<ALIGN>(offset);
         let elem_size = align_offset::<ALIGN>(4);

         let offset = B::read_u32(&buf.chunk()[aligned_header_offset..elem_size]) as usize;
         let length =
             B::read_u32(&buf.chunk()[aligned_header_offset + elem_size..elem_size * 2]) as usize;

         Ok((offset, length))
     }
 }

 impl<const N: usize> Encoder<FixedBytes<N>> for FixedBytes<N> {
     const HEADER_SIZE: usize = N;

     fn encode<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, field_offset: usize) {
         let aligned_offset = A::align(field_offset);
         buffer.resize(aligned_offset + N, 0);
         buffer[aligned_offset..aligned_offset + N].copy_from_slice(&self.0);
     }

     fn decode_header<A: Alignment, E: Endian>(
         bytes: &bytes::Bytes,
         field_offset: usize,
         result: &mut FixedBytes<N>,
     ) -> (usize, usize) {
         let aligned_offset = A::align(field_offset);
         result
             .0
             .copy_from_slice(&bytes[aligned_offset..aligned_offset + N]);
         (0, 0)
     }

     fn decode_body<A: Alignment, E: Endian>(
         bytes: &bytes::Bytes,
         field_offset: usize,
         result: &mut FixedBytes<N>,
     ) {
         Self::decode_header::<A, E>(bytes, field_offset, result);
     }
 }

 macro_rules! impl_evm_fixed {
     ($typ:ty) => {
         impl Encoder<$typ> for $typ {
             const HEADER_SIZE: usize = <$typ>::len_bytes();

             fn encode<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, field_offset: usize) {
                 self.0.encode::<A, E>(buffer, field_offset);
             }

             fn decode_header<A: Alignment, E: Endian>(
                 bytes: &bytes::Bytes,
                 field_offset: usize,
                 result: &mut $typ,
             ) -> (usize, usize) {
                 FixedBytes::<{ Self::HEADER_SIZE }>::decode_header::<A, E>(
                     bytes,
                     field_offset,
                     &mut result.0,
                 )
             }

             fn decode_body<A: Alignment, E: Endian>(
                 bytes: &bytes::Bytes,
                 field_offset: usize,
                 result: &mut $typ,
             ) {
                 Self::decode_header::<A, E>(bytes, field_offset, result);
             }
         }
     };
 }

 impl_evm_fixed!(Address);

 impl<const BITS: usize, const LIMBS: usize> Encoder<Uint<BITS, LIMBS>> for Uint<BITS, LIMBS> {
     const HEADER_SIZE: usize = Self::BYTES;

     fn encode<A: Alignment, E: Endian>(&self, buffer: &mut BytesMut, field_offset: usize) {
         let aligned_offset = A::align(field_offset);
         buffer.resize(aligned_offset + Self::BYTES, 0);
         let bytes = &mut buffer[aligned_offset..aligned_offset + Self::BYTES];
         if E::is_little_endian() {
             bytes.copy_from_slice(&self.as_le_bytes());
         } else {
             bytes.copy_from_slice(&self.to_be_bytes_vec());
         }
     }

     fn decode_header<A: Alignment, E: Endian>(
         bytes: &bytes::Bytes,
         field_offset: usize,
         result: &mut Uint<BITS, LIMBS>,
     ) -> (usize, usize) {
         let aligned_offset = A::align(field_offset);
         let slice = bytes.slice(aligned_offset..aligned_offset + Self::BYTES);

         if E::is_little_endian() {
             *result = Uint::from_le_slice(&slice[..Self::BYTES]);
         } else {
             *result = Uint::from_be_slice(&slice[..Self::BYTES]);
         }
         (0, 0)
     }

     fn decode_body<A: Alignment, E: Endian>(
         bytes: &bytes::Bytes,
         field_offset: usize,
         result: &mut Uint<BITS, LIMBS>,
     ) {
         Self::decode_header::<A, E>(bytes, field_offset, result);
     }
 }

// #[cfg(test)]
// mod tests {
//     use crate::utils::print_buffer_debug;

//     use super::*;
//     use alloy_primitives::{Address, U256};
//     use byteorder::{BigEndian, LittleEndian};

//     // use hex;
//      use hex_literal::hex;

//     // #[test]
//     // fn test_address_encode_decode() {
//     //     let original = Address::from([0x42; 20]);
//     //     let mut buffer = BytesMut::new();

//     //     original.encode::<Align1, LittleEndian>(&mut buffer, 0);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded Address: {}", hex::encode(&encoded));

//     //     let mut decoded = Address::default();
//     //     Address::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

//     //     assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_address_encode_decode_aligned() {
//     //     let original = Address::from([0x42; 20]);
//     //     let mut buffer = BytesMut::new();

//     //     original.encode::<Align8, BigEndian>(&mut buffer, 3);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded Address (Aligned): {}", hex::encode(&encoded));

//     //     let mut decoded = Address::default();
//     //     Address::decode_body::<Align8, BigEndian>(&encoded, 3, &mut decoded);

//     //     assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_uint_encode_decode_le() {
//     //     let original = U256::from(0x1234567890abcdef_u64);
//     //     let mut buffer = BytesMut::new();

//     //     original.encode::<Align4, LittleEndian>(&mut buffer, 0);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded U256 (LE): {}", hex::encode(&encoded));

//     //     let mut decoded = U256::ZERO;
//     //     U256::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

//     //     assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_uint_encode_decode_be() {
//     //     let original = U256::from(0x1234567890abcdef_u64);
//     //     let mut buffer = BytesMut::new();

//     //     original.encode::<Align8, BigEndian>(&mut buffer, 5);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded U256 (BE): {}", hex::encode(&encoded));

//     //     let mut decoded = U256::ZERO;
//     //     U256::decode_body::<Align8, BigEndian>(&encoded, 5, &mut decoded);

//     //     assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_uint_encode_decode_large_number() {
//     //     let value =
//     //         hex_literal::hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
//     //     let original = U256::from_be_bytes(value);
//     //     let mut buffer = BytesMut::new();

//     //     original.encode::<Align1, LittleEndian>(&mut buffer, 0);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded large U256: {}", hex::encode(&encoded));

//     //     let mut decoded = U256::ZERO;
//     //     U256::decode_body::<Align1, LittleEndian>(&encoded, 0, &mut decoded);

//     //     assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_uint_encode_decode_mixed_Endian() {
//     //     let original = U256::from(0x1234567890abcdef_u64);
//     //     let mut buffer = BytesMut::new();

//     //     original.encode::<Align4, BigEndian>(&mut buffer, 0);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded U256 (BE): {}", hex::encode(&encoded));

//     //     let mut decoded = U256::ZERO;
//     //     U256::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

//     //     // This should fail because we're encoding in BE and decoding in LE
//     //     assert_ne!(original, decoded);

//     //     // Correct decoding
//     //     let mut correct_decoded = U256::ZERO;
//     //     U256::decode_body::<Align4, BigEndian>(&encoded, 0, &mut correct_decoded);
//     //     assert_eq!(original, correct_decoded);
//     // }

//     // #[test]
//     // fn test_bytes_a1() {
//     //     let original = Bytes::from_static(b"Hello, World");

//     //     let mut buf = BytesMut::with_capacity(original.size_hint::<32>());

//     //     print_buffer_debug(&buf, Bytes::HEADER_SIZE);
//     //     let is_ok = original.encode::<LittleEndian, 32>(&mut buf, 0);
//     //     assert!(is_ok.is_ok());

//     //     let encoded = buf.freeze();
//     //     println!("Encoded (A1): {}", hex::encode(&encoded));

//     //     assert_eq!(
//     //         hex::encode(&encoded),
//     //         "080000000c00000048656c6c6f2c20576f726c64"
//     //     );
//     //     // let decoded = Bytes::decode::<LittleEndian, 1>(&mut encoded, 0).unwrap();

//     //     // assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_bytes_a4() {
//     //     let original = Bytes::from_static(b"Hello, World");
//     //     let mut buffer = BytesMut::new();
//     //     original.encode::<Align4, LittleEndian>(&mut buffer, 0);

//     //     let encoded = buffer.freeze();
//     //     println!("Encoded (A4): {}", hex::encode(&encoded));

//     //     let mut decoded = Bytes::new();
//     //     Bytes::decode_body::<Align4, LittleEndian>(&encoded, 0, &mut decoded);

//     //     assert_eq!(original, decoded);
//     //     assert_eq!(
//     //         hex::encode(&encoded),
//     //         "080000000c00000048656c6c6f2c20576f726c64"
//     //     );
//     // }

//     // #[test]
//     // fn test_bytes_a8_with_offset() {
//     //     let original = Bytes::from_static(b"Hello");
//     //     let mut buffer = BytesMut::from(&[0xFF, 0xFF, 0xFF][..]);
//     //     buffer.resize(3 + original.size_hint::<8>(), 0);
//     //     let res = original.encode::<LittleEndian, 8>(&mut buffer, 3);
//     //     assert!(res.is_ok());

//     //     let mut encoded = buffer.freeze();
//     //     println!("Encoded (A8 with offset): {}", hex::encode(&encoded));
//     //     assert_eq!(
//     //         hex::encode(&encoded),
//     //         "ffffff00000000001800000000000000050000000000000048656c6c6f"
//     //     );

//     //     // let decoded = Bytes::decode::<LittleEndian, 8>(&mut encoded, 3).unwrap();
//     //     // assert_eq!(original, decoded);
//     // }

//     // #[test]
//     // fn test_buf_mut_advance() {
//     //     let mut buf = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF][..]);

//     //     println!("0. buf: {:#?}", buf);
//     //     println!("buf capacity: {}", buf.capacity());
//     //     println!("buf len: {}", buf.len());
//     //     // buf.advance(3);
//     //     // println!("1. buf: {:#?}", buf);
//     //     // buf.put_u8(1);
//     //     // println!("2. buf: {:#?}", buf);
//     //     fn aaa(buf: &mut impl BufMut) {
//     //         buf.chunk_mut()[0..2].copy_from_slice(b"he");
//     //         /// unsafe { buf.advance_mut(2) };
//     //         unsafe {
//     //             buf.advance_mut(2)
//     //         };
//     //         buf.put_u8(1);
//     //     }
//     //     aaa(&mut buf);
//     //     println!("buf: {:#?}", &buf);

//     //     // unsafe { buf.advance_mut(2) };
//     // }

//     // #[test]
//     // fn test_write_bytes() {
//     //     let original = Bytes::from_static(b"Hello, World");
//     //     let mut buf = BytesMut::with_capacity(128);
//     //     let size_hint = original.size_hint::<32>();
//     //     println!("size_hint: {}", size_hint);

//     //     let result = write_bytes::<BigEndian, 32>(&mut buf, 0, 128, &original);

//     //     assert!(result.is_ok());
//     //     // print_buffer_debug(&buf, Bytes::HEADER_SIZE);

//     //     let mut encoded = buf.freeze();
//     //     println!("Encoded: {}", hex::encode(&encoded));
//     //     println!("encoded: {:?}", encoded.to_vec());

//     //     let decoded = read_bytes::<BigEndian, 32>(&mut encoded, 0).unwrap();
//     //     println!("decoded: {:?}", decoded.to_vec());
//     //     assert_eq!(original, Bytes::from(decoded));
//     // }
// }
