// use core::fmt;

// use byteorder::{BigEndian, ByteOrder, LittleEndian};
// use bytes::Buf;

// use crate::encoder2::{align_offset, Encoder};

// struct EncoderDebugHelper<'a, T: Encoder, B: ByteOrder, BufT: Buf, const ALIGN: usize> {
//     encoder: &'a T,
//     buf: &'a BufT,
//     phantom: std::marker::PhantomData<B>,
// }

// impl<'a, T: Encoder, B: ByteOrder, BufT: Buf, const ALIGN: usize> fmt::Debug
//     for EncoderDebugHelper<'a, T, B, BufT, ALIGN>
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let total_size = self.encoder.size_hint::<ALIGN>();
//         let header_size = T::HEADER_SIZE;
//         let data_size = T::DATA_SIZE;
//         let buf_len = self.buf.remaining();

//         writeln!(f, "Alignment: {}", ALIGN)?;
//         writeln!(f, "Buffer length: {}", buf_len)?;
//         writeln!(f, "Header size: {}", header_size)?;
//         writeln!(f, "Data size: {}", data_size)?;
//         writeln!(f, "Byte order: {}", std::any::type_name::<B>())?;

//         let total_cells = (total_size + 3) / 4;
//         let header_cells = (header_size + 3) / 4;

//         write!(f, "+")?;
//         for _ in 0..total_cells {
//             write!(f, "--------+")?;
//         }
//         writeln!(f)?;

//         write!(f, "|")?;
//         for i in 0..total_cells {
//             if i < header_cells {
//                 write!(f, " header ")?;
//             } else {
//                 write!(f, "  data  ")?;
//             }
//             write!(f, "|")?;
//         }
//         writeln!(f)?;

//         write!(f, "+")?;
//         for _ in 0..total_cells {
//             write!(f, "--------+")?;
//         }
//         writeln!(f)?;

//         write!(f, "|")?;
//         for i in 0..total_size {
//             if i % 4 == 0 && i > 0 {
//                 write!(f, "|")?;
//             }
//             if i < header_size {
//                 write!(f, "  0x{:02X}  ", i)?;
//             } else {
//                 write!(f, "   --   ")?;
//             }
//         }
//         write!(f, "|")?;
//         writeln!(f)
//     }
// }

// pub struct EncoderDebugView<'a, T: Encoder, BufT: Buf> {
//     encoder: &'a T,
//     buf: &'a BufT,
// }

// impl<'a, T: Encoder, BufT: Buf> fmt::Debug for EncoderDebugView<'a, T, BufT> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         writeln!(f, "Encoder Debug View:")?;
//         writeln!(f, "BigEndian:")?;
//         EncoderDebugHelper::<T, BigEndian, BufT, 4> {
//             encoder: self.encoder,
//             buf: self.buf,
//             phantom: std::marker::PhantomData,
//         }
//         .fmt(f)?;
//         writeln!(f)?;
//         writeln!(f, "LittleEndian:")?;
//         EncoderDebugHelper::<T, LittleEndian, BufT, 4> {
//             encoder: self.encoder,
//             buf: self.buf,
//             phantom: std::marker::PhantomData,
//         }
//         .fmt(f)
//     }
// }
// impl<T: Encoder> T {
//     pub fn debug_view<'a, BufT: Buf>(&'a self, buf: &'a BufT) -> EncoderDebugView<'a, T, BufT> {
//         EncoderDebugView { encoder: self, buf }
//     }
// }
// // Пример реализации Encoder для тестирования
// #[derive(Default)]
// struct TestEncoder;

// impl Encoder for TestEncoder {
//     const HEADER_SIZE: usize = 8;
//     const DATA_SIZE: usize = 16;

//     fn size_hint<const ALIGN: usize>(&self) -> usize {
//         align_offset::<ALIGN>(Self::HEADER_SIZE + Self::DATA_SIZE)
//     }

//     fn encode_inner<B: ByteOrder, const ALIGN: usize>(
//         &self,
//         buf: &mut impl bytes::BufMut,
//         offset: usize,
//     ) -> Result<(), crate::encoder2::EncoderError> {
//         Ok(())
//     }

//     fn decode_inner<B: ByteOrder, const ALIGN: usize>(
//         buf: &mut impl bytes::Buf,
//         offset: usize,
//     ) -> Result<Self, crate::encoder2::EncoderError> {
//         Ok(Self)
//     }

//     fn partial_decode<B: ByteOrder, const ALIGN: usize>(
//         buf: &mut impl bytes::Buf,
//         offset: usize,
//     ) -> Result<(usize, usize), crate::encoder2::EncoderError> {
//         Ok((offset, Self::DATA_SIZE))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use bytes::{BufMut, BytesMut};

//     use super::*;

//     #[test]
//     fn test_encoder_debug_view() {
//         let encoder = TestEncoder::default();
//         let mut buffer = BytesMut::with_capacity(32);
//         buffer.put_u32(0x12345678); // Добавляем некоторые данные в буфер
//         buffer.put_u32(0x87654321);

//         let debug_view = format!("{:?}", encoder.debug_view(&buffer));
//         println!("{}", debug_view);

//         // Проверяем, что вывод содержит ожидаемую информацию
//         assert!(debug_view.contains("Alignment: 4"));
//         assert!(debug_view.contains("Buffer length: 8"));
//         assert!(debug_view.contains("Header size: 8"));
//         assert!(debug_view.contains("Data size: 16"));
//         assert!(debug_view.contains("BigEndian"));
//         assert!(debug_view.contains("LittleEndian"));
//     }
// }
