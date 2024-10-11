impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, { true }> for TestStructSmall {
    const HEADER_SIZE: usize = 65;
    const IS_DYNAMIC: bool = true;
    fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let mut current_offset = aligned_offset;
        let mut dynamic_fields_count = 0;
        let mut dynamic_fields_header_size = 0;
        let mut tmp = BytesMut::new();

        // ENCODE STATIC FIELDS, CALCULATE DYNAMIC FIELDS COUNT
        if !<bool as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
            <bool as Encoder<B, ALIGN, { true }>>::encode(
                &self.bool_val,
                &mut tmp,
                current_offset,
            )?;
            current_offset += align_up::<ALIGN>(<bool as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
        } else {
            dynamic_fields_count += 1;
            dynamic_fields_header_size += <bool as Encoder<B, ALIGN, { true }>>::HEADER_SIZE;
        }
        if !<Bytes as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
            <Bytes as Encoder<B, ALIGN, { true }>>::encode(
                &self.bytes_val,
                &mut tmp,
                current_offset,
            )?;
            current_offset +=
                align_up::<ALIGN>(<Bytes as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
        } else {
            dynamic_fields_count += 1;
            dynamic_fields_header_size += <Bytes as Encoder<B, ALIGN, { true }>>::HEADER_SIZE;
        }
        if !<Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
            <Vec<u32> as Encoder<B, ALIGN, { true }>>::encode(
                &self.vec_val,
                &mut tmp,
                current_offset,
            )?;
            current_offset +=
                align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
        } else {
            dynamic_fields_count += 1;
            dynamic_fields_header_size += <Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE;
        }

        if dynamic_fields_count > 0 {
            if tmp.len() < current_offset + dynamic_fields_header_size {
                tmp.resize(current_offset + dynamic_fields_header_size, 0);
            }

            if <bool as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
                <bool as Encoder<B, ALIGN, { true }>>::encode(
                    &self.bool_val,
                    &mut tmp,
                    current_offset,
                )?;
                current_offset +=
                    align_up::<ALIGN>(<bool as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
            }
            if <Bytes as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
                <Bytes as Encoder<B, ALIGN, { true }>>::encode(
                    &self.bytes_val,
                    &mut tmp,
                    current_offset,
                )?;
                current_offset +=
                    align_up::<ALIGN>(<Bytes as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
            }
            if <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
                <Vec<u32> as Encoder<B, ALIGN, { true }>>::encode(
                    &self.vec_val,
                    &mut tmp,
                    current_offset,
                )?;
                current_offset +=
                    align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 32);
        }

        buf.extend_from_slice(&tmp);
        Ok(())
    }
    fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
        let mut current_offset = align_up::<ALIGN>(offset);
        let mut tmp = if false
            || <bool as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC
            || <Bytes as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC
            || <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC
        {
            &buf.chunk()[32..]
        } else {
            buf.chunk()
        };
        let bool_val = if <bool as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
            <bool as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
        } else {
            <bool as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
        };
        current_offset += align_up::<ALIGN>(<bool as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
        let bytes_val = if <Bytes as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
            <Bytes as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
        } else {
            <Bytes as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
        };
        current_offset += align_up::<ALIGN>(<Bytes as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
        let vec_val = if <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
            <Vec<u32> as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
        } else {
            <Vec<u32> as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
        };
        current_offset += align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
        Ok(TestStructSmall {
            bool_val,
            bytes_val,
            vec_val,
        })
    }
    fn partial_decode(buffer: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
        Ok((0, 0))
    }
}

// "00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000000501020304050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001e"
// 0000: 00 00 00 20   ||  000: 032 |
// 0020: 00 00 00 01   ||  032: 001 |
// 0040: 00 00 00 60   ||  064: 096 |
// 0060: 00 00 00 a0   ||  096: 160 |
// 0080: 00 00 00 05   ||  128: 005 |
// 00a0: 00 00 00 00   ||  160: 000 |
// 00c0: 00 00 00 03   ||  192: 003 |
// 00e0: 00 00 00 0a   ||  224: 010 |
// 0100: 00 00 00 14   ||  256: 020 |
// 0120: 00 00 00 1e   ||  288: 030 |





// // Recursive expansion of Codec macro
// // ===================================

// impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, { true }> for TestNestedStruct {
//     const HEADER_SIZE: usize = 0
//         + <TestStructSmall as Encoder<B, ALIGN, { true }>>::HEADER_SIZE
//         + <Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE;
//     const IS_DYNAMIC: bool = false
//         || <TestStructSmall as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC
//         || <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC;

//     fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
//         let aligned_offset = align_up::<ALIGN>(offset);
//         let mut dynamic_fields_count = 0;
//         let mut dynamic_fields_header_size = 0;
//         let is_dynamic = <Self as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC;

//         // Create a new BytesMut for temporary storage
//         let mut tmp = BytesMut::new();

//         let cur_len = buf.len();

//         // Write the dynamic struct offset if necessary
//         if is_dynamic {
//             write_u32_aligned::<B, ALIGN>(buf, aligned_offset, (cur_len + 32) as u32);
//         }

//         let mut current_offset = 0; // Start at 0 for tmp buffer

//         // ENCODE STATIC FIELDS, CALCULATE DYNAMIC FIELDS COUNT
//         if !<TestStructSmall as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
//             <TestStructSmall as Encoder<B, ALIGN, { true }>>::encode(
//                 &self.nested_struct,
//                 &mut tmp,
//                 current_offset,
//             )?;
//             current_offset +=
//                 align_up::<ALIGN>(<TestStructSmall as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
//         } else {
//             dynamic_fields_count += 1;
//             dynamic_fields_header_size +=
//                 <TestStructSmall as Encoder<B, ALIGN, { true }>>::HEADER_SIZE;
//         }
//         if !<Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
//             <Vec<u32> as Encoder<B, ALIGN, { true }>>::encode(
//                 &self.vec_val,
//                 &mut tmp,
//                 current_offset,
//             )?;
//             current_offset +=
//                 align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
//         } else {
//             dynamic_fields_count += 1;
//             dynamic_fields_header_size += <Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE;
//         }

//         // ENCODE DYNAMIC FIELDS
//         if dynamic_fields_count > 0 {
//             if tmp.len() < current_offset + dynamic_fields_count * 32 {
//                 tmp.resize(current_offset + dynamic_fields_count * 32, 0);
//             }

//             if <TestStructSmall as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
//                 <TestStructSmall as Encoder<B, ALIGN, { true }>>::encode(
//                     &self.nested_struct,
//                     &mut tmp,
//                     current_offset,
//                 )?;
//                 current_offset += align_up::<ALIGN>(
//                     <TestStructSmall as Encoder<B, ALIGN, { true }>>::HEADER_SIZE,
//                 );
//             }
//             if <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
//                 <Vec<u32> as Encoder<B, ALIGN, { true }>>::encode(
//                     &self.vec_val,
//                     &mut tmp,
//                     current_offset,
//                 )?;
//                 current_offset +=
//                     align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
//             }
//         }

//         // Append the temporary buffer to the main buffer
//         buf.extend_from_slice(&tmp);

//         Ok(())
//     }

//     fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
//         let mut current_offset = align_up::<ALIGN>(offset);
//         let mut tmp = if false
//             || <TestStructSmall as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC
//             || <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC
//         {
//             &buf.chunk()[32..]
//         } else {
//             buf.chunk()
//         };
//         let nested_struct = if <TestStructSmall as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
//             <TestStructSmall as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
//         } else {
//             <TestStructSmall as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
//         };
//         current_offset +=
//             align_up::<ALIGN>(<TestStructSmall as Encoder<B, ALIGN, { true }>>::HEADER_SIZE);
//         let vec_val = if <Vec<u32> as Encoder<B, ALIGN, { true }>>::IS_DYNAMIC {
//             <Vec<u32> as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
//         } else {
//             <Vec<u32> as Encoder<B, ALIGN, { true }>>::decode(&mut tmp, current_offset)?
//         };
//         current_offset += align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { true
// }>>::HEADER_SIZE);         Ok(TestNestedStruct {
//             nested_struct,
//             vec_val,
//         })
//     }
//     fn partial_decode(buffer: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
//         Ok((0, 0))
//     }
// }
// impl<B: ByteOrder, const ALIGN: usize> Encoder<B, ALIGN, { false }> for TestNestedStruct {
//     const HEADER_SIZE: usize = 0
//         + <TestStructSmall as Encoder<B, ALIGN, { false }>>::HEADER_SIZE
//         + <Vec<u32> as Encoder<B, ALIGN, { false }>>::HEADER_SIZE;
//     const IS_DYNAMIC: bool = false
//         || <TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC
//         || <Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC;
//     fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
//         let aligned_offset = align_up::<ALIGN>(offset);
//         let mut current_offset = aligned_offset;
//         let mut tmp = BytesMut::new();
//         if !<TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC {
//             <TestStructSmall as Encoder<B, ALIGN, { false }>>::encode(
//                 &self.nested_struct,
//                 &mut tmp,
//                 current_offset,
//             )?;
//             current_offset +=
//                 align_up::<ALIGN>(<TestStructSmall as Encoder<B, ALIGN, { false
// }>>::HEADER_SIZE);         }
//         if !<Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC {
//             <Vec<u32> as Encoder<B, ALIGN, { false }>>::encode(
//                 &self.vec_val,
//                 &mut tmp,
//                 current_offset,
//             )?;
//             current_offset +=
//                 align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { false }>>::HEADER_SIZE);
//         }
//         if false
//             || <TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC
//             || <Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC
//         {
//             let dynamic_fields_header_size = 0
//                 + (<TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC as usize
//                     * <TestStructSmall as Encoder<B, ALIGN, { false }>>::HEADER_SIZE)
//                 + (<Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC as usize
//                     * <Vec<u32> as Encoder<B, ALIGN, { false }>>::HEADER_SIZE);
//             if tmp.len() < current_offset + dynamic_fields_header_size {
//                 tmp.resize(current_offset + dynamic_fields_header_size, 0);
//             }
//             if <TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC {
//                 <TestStructSmall as Encoder<B, ALIGN, { false }>>::encode(
//                     &self.nested_struct,
//                     &mut tmp,
//                     current_offset,
//                 )?;
//                 current_offset += align_up::<ALIGN>(
//                     <TestStructSmall as Encoder<B, ALIGN, { false }>>::HEADER_SIZE,
//                 );
//             }
//             if <Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC {
//                 <Vec<u32> as Encoder<B, ALIGN, { false }>>::encode(
//                     &self.vec_val,
//                     &mut tmp,
//                     current_offset,
//                 )?;
//                 current_offset +=
//                     align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { false }>>::HEADER_SIZE);
//             }
//             write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 32);
//         }
//         buf.extend_from_slice(&tmp);
//         Ok(())
//     }
//     fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
//         let mut current_offset = align_up::<ALIGN>(offset);
//         let mut tmp = if false
//             || <TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC
//             || <Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC
//         {
//             &buf.chunk()[32..]
//         } else {
//             buf.chunk()
//         };
//         let nested_struct = if <TestStructSmall as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC {
//             <TestStructSmall as Encoder<B, ALIGN, { false }>>::decode(&mut tmp, current_offset)?
//         } else {
//             <TestStructSmall as Encoder<B, ALIGN, { false }>>::decode(&mut tmp, current_offset)?
//         };
//         current_offset +=
//             align_up::<ALIGN>(<TestStructSmall as Encoder<B, ALIGN, { false }>>::HEADER_SIZE);
//         let vec_val = if <Vec<u32> as Encoder<B, ALIGN, { false }>>::IS_DYNAMIC {
//             <Vec<u32> as Encoder<B, ALIGN, { false }>>::decode(&mut tmp, current_offset)?
//         } else {
//             <Vec<u32> as Encoder<B, ALIGN, { false }>>::decode(&mut tmp, current_offset)?
//         };
//         current_offset +=
//             align_up::<ALIGN>(<Vec<u32> as Encoder<B, ALIGN, { false }>>::HEADER_SIZE);
//         Ok(TestNestedStruct {
//             nested_struct,
//             vec_val,
//         })
//     }
//     fn partial_decode(buffer: &impl Buf, offset: usize) -> Result<(usize, usize), CodecError> {
//         Ok((0, 0))
//     }
// }
