#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod bytes {
    extern crate alloc;
    use crate::{
        encoder::{align_up, read_u32_aligned, write_u32_aligned},
        error::{CodecError, DecodingError},
    };
    use byteorder::ByteOrder;
    use bytes::{Buf, Bytes, BytesMut};
    use core::mem;
    /// Write bytes in Solidity compatible format
    pub fn write_bytes_solidity<B: ByteOrder, const ALIGN: usize>(
        buf: &mut BytesMut,
        header_offset: usize,
        data: &[u8],
        elements: u32,
    ) -> usize {
        let aligned_offset = align_up::<ALIGN>(header_offset);
        if buf.len() < aligned_offset {
            buf.resize(aligned_offset, 0);
        }
        let data_offset = buf.len();
        write_u32_aligned::<B, ALIGN>(buf, data_offset, elements as u32);
        buf.extend_from_slice(data);
        buf.len() - data_offset
    }
    pub fn write_bytes_solidity2<B: ByteOrder, const ALIGN: usize>(
        buf: &mut BytesMut,
        data: &[u8],
        elements: u32,
    ) -> usize {
        let data_offset = buf.len();
        write_u32_aligned::<B, ALIGN>(buf, data_offset, elements as u32);
        buf.extend_from_slice(data);
        buf.len() - data_offset
    }
    /// Write bytes in WASM compatible format
    pub fn write_bytes_wasm<B: ByteOrder, const ALIGN: usize>(
        buf: &mut BytesMut,
        header_offset: usize,
        data: &[u8],
    ) -> usize {
        let aligned_offset = align_up::<ALIGN>(header_offset);
        let aligned_elem_size = align_up::<ALIGN>(mem::size_of::<u32>());
        let aligned_header_size = aligned_elem_size * 2;
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }
        let data_offset = buf.len();
        write_u32_aligned::<B, ALIGN>(buf, aligned_offset, data_offset as u32);
        write_u32_aligned::<
            B,
            ALIGN,
        >(buf, aligned_offset + aligned_elem_size, data.len() as u32);
        buf.extend_from_slice(data);
        buf.len() - data_offset
    }
    /// Universal function to write bytes in Solidity or WASM compatible format
    pub fn write_bytes<B, const ALIGN: usize, const SOL_MODE: bool>(
        buf: &mut BytesMut,
        header_offset: usize,
        data: &[u8],
        elements: u32,
    ) -> usize
    where
        B: ByteOrder,
    {
        match SOL_MODE {
            true => write_bytes_solidity::<B, ALIGN>(buf, header_offset, data, elements),
            false => write_bytes_wasm::<B, ALIGN>(buf, header_offset, data),
        }
    }
    pub fn read_bytes_header_wasm<B: ByteOrder, const ALIGN: usize>(
        buffer: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let aligned_elem_size = align_up::<ALIGN>(mem::size_of::<u32>());
        if buffer.remaining() < aligned_offset + aligned_elem_size * 2 {
            return Err(
                CodecError::Decoding(DecodingError::BufferTooSmall {
                    expected: aligned_offset + aligned_elem_size * 2,
                    found: buffer.remaining(),
                    msg: "buffer too small to read bytes header".to_string(),
                }),
            );
        }
        let data_offset = read_u32_aligned::<B, ALIGN>(buffer, aligned_offset)? as usize;
        let data_len = read_u32_aligned::<
            B,
            ALIGN,
        >(buffer, aligned_offset + aligned_elem_size)? as usize;
        Ok((data_offset, data_len))
    }
    pub fn read_bytes_header_solidity<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        let aligned_offset = align_up::<ALIGN>(offset);
        let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
        let element_offset = data_offset + ALIGN;
        let element_len = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;
        Ok((element_offset, element_len))
    }
    /// Reads the header of the bytes data in Solidity or WASM compatible format
    /// Returns the offset and size of the data
    pub fn read_bytes_header<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        match SOL_MODE {
            true => read_bytes_header_solidity::<B, ALIGN>(buf, offset),
            false => read_bytes_header_wasm::<B, ALIGN>(buf, offset),
        }
    }
    pub fn read_bytes_wasm<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Bytes, CodecError> {
        let (data_offset, data_len) = read_bytes_header_wasm::<B, ALIGN>(buf, offset)?;
        let data = buf.chunk()[data_offset..data_offset + data_len].to_vec();
        Ok(Bytes::from(data))
    }
    pub fn read_bytes<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<Bytes, CodecError> {
        let (data_offset, data_len) = read_bytes_header::<
            B,
            ALIGN,
            SOL_MODE,
        >(buf, offset)?;
        let data = if SOL_MODE {
            buf.chunk()[data_offset..data_offset + data_len].to_vec()
        } else {
            buf.chunk()[data_offset..].to_vec()
        };
        Ok(Bytes::from(data))
    }
}
pub mod empty {
    use crate::{
        encoder::{align_up, read_u32_aligned, write_u32_aligned, Encoder},
        error::{CodecError, DecodingError},
    };
    use byteorder::ByteOrder;
    use bytes::{Buf, BytesMut};
    pub struct EmptyVec;
    #[automatically_derived]
    impl ::core::fmt::Debug for EmptyVec {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(f, "EmptyVec")
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for EmptyVec {
        #[inline]
        fn clone(&self) -> EmptyVec {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for EmptyVec {}
    #[automatically_derived]
    impl ::core::default::Default for EmptyVec {
        #[inline]
        fn default() -> EmptyVec {
            EmptyVec {}
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for EmptyVec {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for EmptyVec {
        #[inline]
        fn eq(&self, other: &EmptyVec) -> bool {
            true
        }
    }
    impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for EmptyVec {
        const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_elem_size = align_up::<ALIGN>(4);
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 0);
            write_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_elem_size, (aligned_elem_size * 3) as u32);
            write_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_elem_size * 2, 0);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_elem_size = align_up::<ALIGN>(4);
            if buf.remaining()
                < aligned_offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE
            {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset
                            + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE,
                        found: buf.remaining(),
                        msg: "failed to decode EmptyVec".to_string(),
                    }),
                );
            }
            let count = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?;
            if count != 0 {
                return Err(
                    CodecError::Decoding(
                        DecodingError::InvalidData(
                            "EmptyVec must have count of 0".to_string(),
                        ),
                    ),
                );
            }
            let data_offset = read_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_elem_size)? as usize;
            let data_length = read_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_elem_size * 2)? as usize;
            if data_offset != <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE
                || data_length != 0
            {
                return Err(
                    CodecError::Decoding(
                        DecodingError::InvalidData(
                            "Invalid offset or length for EmptyVec".to_string(),
                        ),
                    ),
                );
            }
            Ok(EmptyVec)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_elem_size = align_up::<ALIGN>(4);
            if buf.remaining()
                < aligned_offset + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE
            {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset
                            + <Self as Encoder<B, ALIGN, false>>::HEADER_SIZE,
                        found: buf.remaining(),
                        msg: "failed to partially decode EmptyVec".to_string(),
                    }),
                );
            }
            let count = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?;
            if count != 0 {
                return Err(
                    CodecError::Decoding(
                        DecodingError::InvalidData(
                            "EmptyVec must have count of 0".to_string(),
                        ),
                    ),
                );
            }
            let data_offset = read_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_elem_size)? as usize;
            let data_length = read_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_elem_size * 2)? as usize;
            Ok((data_offset, data_length))
        }
    }
    impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for EmptyVec {
        const HEADER_SIZE: usize = 32;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            write_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset, (aligned_offset + 32) as u32);
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset + 32, 0);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            if buf.remaining() < aligned_offset + 32 {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + 32,
                        found: buf.remaining(),
                        msg: "failed to decode EmptyVec".to_string(),
                    }),
                );
            }
            let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?
                as usize;
            let length = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;
            if length != 0 {
                return Err(
                    CodecError::Decoding(
                        DecodingError::InvalidData(
                            "EmptyVec must have length of 0".to_string(),
                        ),
                    ),
                );
            }
            Ok(EmptyVec)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            if buf.remaining() < aligned_offset + 32 {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + 32,
                        found: buf.remaining(),
                        msg: "failed to partially decode EmptyVec".to_string(),
                    }),
                );
            }
            let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?
                as usize;
            let length = read_u32_aligned::<B, ALIGN>(buf, data_offset)? as usize;
            Ok((data_offset, length))
        }
    }
}
pub mod encoder {
    use crate::error::CodecError;
    use byteorder::{ByteOrder, BE, LE};
    use bytes::{Buf, Bytes, BytesMut};
    use std::marker::PhantomData;
    /// Trait for encoding and decoding values with specific byte order, alignment, and mode.
    ///
    /// # Type Parameters
    /// - `B`: The byte order used for encoding/decoding.
    /// - `ALIGN`: The alignment requirement for the encoded data.
    /// - `SOL_MODE`: A boolean flag indicating whether Solidity-compatible mode is enabled.
    pub trait Encoder<B: ByteOrder, const ALIGN: usize, const SOL_MODE: bool>: Sized {
        /// Returns the header size for this encoder.
        const HEADER_SIZE: usize;
        const IS_DYNAMIC: bool;
        /// Encodes the value into the given buffer at the specified offset.
        ///
        /// # Arguments
        /// * `buf` - The buffer to encode into.
        /// * `offset` - The starting offset in the buffer for encoding.
        ///
        /// # Returns
        /// `Ok(())` if encoding was successful, or an error if encoding failed.
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError>;
        /// Decodes a value from the given buffer starting at the specified offset.
        ///
        /// # Arguments
        /// * `buf` - The buffer to decode from.
        /// * `offset` - The starting offset in the buffer for decoding.
        ///
        /// # Returns
        /// The decoded value if successful, or an error if decoding failed.
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError>;
        /// Partially decodes the header to determine the length and offset of the encoded data.
        ///
        /// # Arguments
        /// * `buf` - The buffer to decode from.
        /// * `offset` - The starting offset in the buffer for decoding.
        ///
        /// # Returns
        /// A tuple `(data_offset, data_length)` if successful, or an error if decoding failed.
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError>;
        /// Calculates the number of bytes needed to encode the value.
        ///
        /// This includes the header size and any additional space needed for alignment.
        /// The default implementation aligns the header size to the specified alignment.
        fn size_hint(&self) -> usize {
            align_up::<ALIGN>(Self::HEADER_SIZE)
        }
    }
    pub struct SolidityABI<T>(PhantomData<T>);
    impl<T> SolidityABI<T>
    where
        T: Encoder<BE, 32, true>,
    {
        pub fn encode(
            value: &T,
            buf: &mut BytesMut,
            offset: usize,
        ) -> Result<(), CodecError> {
            value.encode(buf, offset)
        }
        pub fn decode(buf: &impl Buf, offset: usize) -> Result<T, CodecError> {
            T::decode(buf, offset)
        }
        pub fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            T::partial_decode(buf, offset)
        }
        pub fn size_hint(value: &T) -> usize {
            value.size_hint()
        }
    }
    pub struct WasmABI<T>(PhantomData<T>);
    impl<T> WasmABI<T>
    where
        T: Encoder<LE, 4, false>,
    {
        pub fn encode(
            value: &T,
            buf: &mut BytesMut,
            offset: usize,
        ) -> Result<(), CodecError> {
            value.encode(buf, offset)
        }
        pub fn decode(buf: &impl Buf, offset: usize) -> Result<T, CodecError> {
            T::decode(buf, offset)
        }
        pub fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            T::partial_decode(buf, offset)
        }
        pub fn size_hint(value: &T) -> usize {
            value.size_hint()
        }
    }
    pub fn is_big_endian<B: ByteOrder>() -> bool {
        B::read_u16(&[0x12, 0x34]) == 0x1234
    }
    /// Rounds up the given offset to the nearest multiple of ALIGN.
    /// ALIGN must be a power of two.
    #[inline]
    pub const fn align_up<const ALIGN: usize>(offset: usize) -> usize {
        (offset + ALIGN - 1) & !(ALIGN - 1)
    }
    /// Aligns the source bytes to the specified alignment.
    pub fn align<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        src: &[u8],
    ) -> Bytes {
        let aligned_src_len = align_up::<ALIGN>(src.len());
        let aligned_total_size = aligned_src_len.max(ALIGN);
        let mut aligned = BytesMut::zeroed(aligned_total_size);
        if is_big_endian::<B>() {
            let start = aligned_total_size - src.len();
            aligned[start..].copy_from_slice(src);
        } else {
            aligned[..src.len()].copy_from_slice(src);
        }
        aligned.freeze()
    }
    pub fn write_u32_aligned<B: ByteOrder, const ALIGN: usize>(
        buf: &mut BytesMut,
        offset: usize,
        value: u32,
    ) {
        let aligned_value_size = align_up::<ALIGN>(4);
        if buf.len() < offset + aligned_value_size {
            buf.resize(offset + aligned_value_size, 0);
        }
        if is_big_endian::<B>() {
            let start = offset + aligned_value_size - 4;
            B::write_u32(&mut buf[start..], value);
        } else {
            B::write_u32(&mut buf[offset..offset + 4], value);
        }
    }
    pub fn read_u32_aligned<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<u32, CodecError> {
        let aligned_value_size = align_up::<ALIGN>(4);
        let end_offset = offset
            .checked_add(aligned_value_size)
            .ok_or_else(|| {
                CodecError::Decoding(crate::error::DecodingError::BufferOverflow {
                    msg: "Overflow occurred when calculating end offset while reading aligned u32"
                        .to_string(),
                })
            })?;
        if buf.remaining() < end_offset {
            return Err(
                CodecError::Decoding(crate::error::DecodingError::BufferTooSmall {
                    expected: end_offset,
                    found: buf.remaining(),
                    msg: "Buffer underflow occurred while reading aligned u32"
                        .to_string(),
                }),
            );
        }
        if is_big_endian::<B>() {
            Ok(B::read_u32(&buf.chunk()[end_offset - 4..end_offset]))
        } else {
            Ok(B::read_u32(&buf.chunk()[offset..offset + 4]))
        }
    }
    pub fn read_u32_aligned1<B: ByteOrder, const ALIGN: usize>(
        buf: &impl Buf,
        offset: usize,
    ) -> Result<u32, CodecError> {
        let aligned_value_size = align_up::<ALIGN>(4);
        let end_offset = offset
            .checked_add(aligned_value_size)
            .ok_or_else(|| {
                CodecError::Decoding(crate::error::DecodingError::BufferOverflow {
                    msg: "Overflow occurred when calculating end offset while reading aligned u32"
                        .to_string(),
                })
            })?;
        if buf.remaining() < end_offset {
            return Err(
                CodecError::Decoding(crate::error::DecodingError::BufferTooSmall {
                    expected: end_offset,
                    found: buf.remaining(),
                    msg: "Buffer underflow occurred while reading aligned u32"
                        .to_string(),
                }),
            );
        }
        if is_big_endian::<B>() {
            Ok(B::read_u32(&buf.chunk()[end_offset - 4..end_offset]))
        } else {
            Ok(B::read_u32(&buf.chunk()[offset..offset + 4]))
        }
    }
    /// Returns a mutable slice of the buffer at the specified offset, aligned to the specified
    /// alignment. This slice is guaranteed to be large enough to hold the value of value_size.
    pub fn get_aligned_slice<B: ByteOrder, const ALIGN: usize>(
        buf: &mut BytesMut,
        offset: usize,
        value_size: usize,
    ) -> &mut [u8] {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size = align_up::<ALIGN>(ALIGN.max(value_size));
        if buf.len() < aligned_offset + word_size {
            buf.resize(aligned_offset + word_size, 0);
        }
        let write_offset = if is_big_endian::<B>() {
            aligned_offset + word_size - value_size
        } else {
            aligned_offset
        };
        &mut buf[write_offset..write_offset + value_size]
    }
    pub fn get_aligned_indices<B: ByteOrder, const ALIGN: usize>(
        offset: usize,
        value_size: usize,
    ) -> (usize, usize) {
        let aligned_offset = align_up::<ALIGN>(offset);
        let word_size = align_up::<ALIGN>(ALIGN.max(value_size));
        {
            ::std::io::_print(
                format_args!(
                    "aligned_offset: {0}, word_size: {1}\n",
                    aligned_offset,
                    word_size,
                ),
            );
        };
        {
            ::std::io::_print(format_args!("value_size: {0}\n", value_size));
        };
        let write_offset = if is_big_endian::<B>() {
            aligned_offset + word_size - value_size
        } else {
            aligned_offset
        };
        (write_offset, write_offset + value_size)
    }
}
pub mod error {
    use thiserror::Error;
    pub enum CodecError {
        #[error("Overflow error")]
        Overflow,
        #[error("Encoding error: {0}")]
        Encoding(#[from] EncodingError),
        #[error("Decoding error: {0}")]
        Decoding(#[from] DecodingError),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for CodecError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                CodecError::Overflow => ::core::fmt::Formatter::write_str(f, "Overflow"),
                CodecError::Encoding(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Encoding",
                        &__self_0,
                    )
                }
                CodecError::Decoding(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Decoding",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for CodecError {
        fn source(&self) -> ::core::option::Option<&(dyn std::error::Error + 'static)> {
            use thiserror::__private::AsDynError as _;
            #[allow(deprecated)]
            match self {
                CodecError::Overflow { .. } => ::core::option::Option::None,
                CodecError::Encoding { 0: source, .. } => {
                    ::core::option::Option::Some(source.as_dyn_error())
                }
                CodecError::Decoding { 0: source, .. } => {
                    ::core::option::Option::Some(source.as_dyn_error())
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl ::core::fmt::Display for CodecError {
        fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            use thiserror::__private::AsDisplay as _;
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                CodecError::Overflow {} => __formatter.write_str("Overflow error"),
                CodecError::Encoding(_0) => {
                    __formatter
                        .write_fmt(format_args!("Encoding error: {0}", _0.as_display()))
                }
                CodecError::Decoding(_0) => {
                    __formatter
                        .write_fmt(format_args!("Decoding error: {0}", _0.as_display()))
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl ::core::convert::From<EncodingError> for CodecError {
        #[allow(deprecated)]
        fn from(source: EncodingError) -> Self {
            CodecError::Encoding { 0: source }
        }
    }
    #[allow(unused_qualifications)]
    impl ::core::convert::From<DecodingError> for CodecError {
        #[allow(deprecated)]
        fn from(source: DecodingError) -> Self {
            CodecError::Decoding { 0: source }
        }
    }
    pub enum EncodingError {
        #[error(
            "Not enough space in the buf: required {required} bytes, but only {available} bytes available. {details}"
        )]
        BufferTooSmall { required: usize, available: usize, details: String },
        #[error("Invalid data provided for encoding: {0}")]
        InvalidInputData(String),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for EncodingError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                EncodingError::BufferTooSmall {
                    required: __self_0,
                    available: __self_1,
                    details: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "BufferTooSmall",
                        "required",
                        __self_0,
                        "available",
                        __self_1,
                        "details",
                        &__self_2,
                    )
                }
                EncodingError::InvalidInputData(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "InvalidInputData",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for EncodingError {}
    #[allow(unused_qualifications)]
    impl ::core::fmt::Display for EncodingError {
        fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            use thiserror::__private::AsDisplay as _;
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                EncodingError::BufferTooSmall { required, available, details } => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Not enough space in the buf: required {0} bytes, but only {1} bytes available. {2}",
                                required.as_display(),
                                available.as_display(),
                                details.as_display(),
                            ),
                        )
                }
                EncodingError::InvalidInputData(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Invalid data provided for encoding: {0}",
                                _0.as_display(),
                            ),
                        )
                }
            }
        }
    }
    pub enum DecodingError {
        #[error("Invalid data encountered during decoding: {0}")]
        InvalidData(String),
        #[error(
            "Not enough data in the buf: expected at least {expected} bytes, found {found}"
        )]
        BufferTooSmall { expected: usize, found: usize, msg: String },
        #[error("Buffer overflow: {msg}")]
        BufferOverflow { msg: String },
        #[error("Unexpected end of buf")]
        UnexpectedEof,
        #[error("Overflow error")]
        Overflow,
        #[error("Parsing error: {0}")]
        ParseError(String),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for DecodingError {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                DecodingError::InvalidData(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "InvalidData",
                        &__self_0,
                    )
                }
                DecodingError::BufferTooSmall {
                    expected: __self_0,
                    found: __self_1,
                    msg: __self_2,
                } => {
                    ::core::fmt::Formatter::debug_struct_field3_finish(
                        f,
                        "BufferTooSmall",
                        "expected",
                        __self_0,
                        "found",
                        __self_1,
                        "msg",
                        &__self_2,
                    )
                }
                DecodingError::BufferOverflow { msg: __self_0 } => {
                    ::core::fmt::Formatter::debug_struct_field1_finish(
                        f,
                        "BufferOverflow",
                        "msg",
                        &__self_0,
                    )
                }
                DecodingError::UnexpectedEof => {
                    ::core::fmt::Formatter::write_str(f, "UnexpectedEof")
                }
                DecodingError::Overflow => {
                    ::core::fmt::Formatter::write_str(f, "Overflow")
                }
                DecodingError::ParseError(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "ParseError",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[allow(unused_qualifications)]
    impl std::error::Error for DecodingError {}
    #[allow(unused_qualifications)]
    impl ::core::fmt::Display for DecodingError {
        fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            use thiserror::__private::AsDisplay as _;
            #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
            match self {
                DecodingError::InvalidData(_0) => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Invalid data encountered during decoding: {0}",
                                _0.as_display(),
                            ),
                        )
                }
                DecodingError::BufferTooSmall { expected, found, msg } => {
                    __formatter
                        .write_fmt(
                            format_args!(
                                "Not enough data in the buf: expected at least {0} bytes, found {1}",
                                expected.as_display(),
                                found.as_display(),
                            ),
                        )
                }
                DecodingError::BufferOverflow { msg } => {
                    __formatter
                        .write_fmt(
                            format_args!("Buffer overflow: {0}", msg.as_display()),
                        )
                }
                DecodingError::UnexpectedEof {} => {
                    __formatter.write_str("Unexpected end of buf")
                }
                DecodingError::Overflow {} => __formatter.write_str("Overflow error"),
                DecodingError::ParseError(_0) => {
                    __formatter
                        .write_fmt(format_args!("Parsing error: {0}", _0.as_display()))
                }
            }
        }
    }
}
pub mod evm {
    use crate::{
        bytes::{read_bytes, read_bytes_header_solidity, write_bytes},
        encoder::{
            align_up, get_aligned_slice, is_big_endian, read_u32_aligned,
            write_u32_aligned, Encoder,
        },
        error::{CodecError, DecodingError},
    };
    use alloy_primitives::{Address, Bytes, FixedBytes, Uint};
    use byteorder::ByteOrder;
    use bytes::{Buf, BytesMut};
    use std::usize;
    impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for Bytes {
        const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let elem_size = align_up::<ALIGN>(4);
            if buf.len() < aligned_offset + elem_size {
                buf.resize(aligned_offset + elem_size, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, buf.len() as u32);
            let _ = write_bytes::<
                B,
                ALIGN,
                true,
            >(buf, aligned_offset, self, self.len() as u32);
            if buf.len() % ALIGN != 0 {
                let padding = ALIGN - (buf.len() % ALIGN);
                buf.resize(buf.len() + padding, 0);
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let data = read_bytes::<B, ALIGN, true>(buf, offset)?;
            Ok(Self::from(data))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            read_bytes_header_solidity::<B, ALIGN>(buf, offset)
        }
    }
    impl<B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for Bytes {
        const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 2;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let elem_size = align_up::<ALIGN>(4);
            if buf.len() < aligned_offset + elem_size {
                buf.resize(aligned_offset + elem_size, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, buf.len() as u32);
            let _ = write_bytes::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset, self, self.len() as u32);
            if buf.len() % ALIGN != 0 {
                let padding = ALIGN - (buf.len() % ALIGN);
                buf.resize(buf.len() + padding, 0);
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let (data_offset, _data_size) = <Self as Encoder<
                B,
                { ALIGN },
                false,
            >>::partial_decode(buf, aligned_offset)?;
            let data = read_bytes::<B, ALIGN, false>(buf, data_offset)?;
            Ok(Self::from(data))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let data_offset = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)?
                as usize;
            let data_size = read_u32_aligned::<B, ALIGN>(buf, aligned_offset + 4)?
                as usize;
            Ok((data_offset, data_size))
        }
    }
    impl<const N: usize, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false>
    for FixedBytes<N> {
        const HEADER_SIZE: usize = N;
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let slice = get_aligned_slice::<B, ALIGN>(buf, aligned_offset, N);
            slice.copy_from_slice(self.as_ref());
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            if buf.remaining() < aligned_offset + N {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + N,
                        found: buf.remaining(),
                        msg: "Buffer too small to decode FixedBytes".to_string(),
                    }),
                );
            }
            let data = buf.chunk()[aligned_offset..aligned_offset + N].to_vec();
            Ok(FixedBytes::from_slice(&data))
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            Ok((aligned_offset, N))
        }
    }
    impl<const N: usize, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true>
    for FixedBytes<N> {
        const HEADER_SIZE: usize = 32;
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<32>(offset);
            let slice = get_aligned_slice::<B, 32>(buf, aligned_offset, 32);
            slice[..N].copy_from_slice(self.as_ref());
            slice[N..].fill(0);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<32>(offset);
            if buf.remaining() < aligned_offset + 32 {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + 32,
                        found: buf.remaining(),
                        msg: "Buffer too small to decode FixedBytes".to_string(),
                    }),
                );
            }
            let data = buf.chunk()[aligned_offset..aligned_offset + N].to_vec();
            Ok(FixedBytes::from_slice(&data))
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<32>(offset);
            Ok((aligned_offset, 32))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for Address {
        const HEADER_SIZE: usize = if SOL_MODE { 32 } else { <Address>::len_bytes() };
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = if SOL_MODE {
                align_up::<32>(offset)
            } else {
                align_up::<ALIGN>(offset)
            };
            let word_size = if SOL_MODE {
                32
            } else {
                align_up::<
                    ALIGN,
                >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
            };
            let slice = get_aligned_slice::<
                B,
                { ALIGN },
            >(buf, aligned_offset, word_size);
            let bytes: &[u8] = self.0.as_ref();
            if SOL_MODE {
                slice[word_size - Self::len_bytes()..].copy_from_slice(bytes);
                slice[..word_size - Self::len_bytes()].fill(0);
            } else if is_big_endian::<B>() {
                slice[word_size - Self::len_bytes()..].copy_from_slice(bytes);
            } else {
                slice[..Self::len_bytes()].copy_from_slice(bytes);
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = if SOL_MODE {
                align_up::<32>(offset)
            } else {
                align_up::<ALIGN>(offset)
            };
            let word_size = if SOL_MODE {
                32
            } else {
                align_up::<
                    ALIGN,
                >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
            };
            if buf.remaining() < aligned_offset + word_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + word_size,
                        found: buf.remaining(),
                        msg: {
                            let res = ::alloc::fmt::format(
                                format_args!("buf too small to read aligned {0}", "Address"),
                            );
                            res
                        },
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..aligned_offset + word_size];
            let inner = if SOL_MODE || is_big_endian::<B>() {
                FixedBytes::<
                    { Self::len_bytes() },
                >::from_slice(&chunk[word_size - Self::len_bytes()..])
            } else {
                FixedBytes::<
                    { Self::len_bytes() },
                >::from_slice(&chunk[..Self::len_bytes()])
            };
            Ok(Self(inner))
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = if SOL_MODE {
                align_up::<32>(offset)
            } else {
                align_up::<ALIGN>(offset)
            };
            let word_size = if SOL_MODE {
                32
            } else {
                align_up::<
                    ALIGN,
                >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
            };
            Ok((aligned_offset, word_size))
        }
    }
    impl<
        const BITS: usize,
        const LIMBS: usize,
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for Uint<BITS, LIMBS> {
        const HEADER_SIZE: usize = if SOL_MODE { 32 } else { Self::BYTES };
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = if SOL_MODE {
                align_up::<32>(offset)
            } else {
                align_up::<ALIGN>(offset)
            };
            let word_size = if SOL_MODE { 32 } else { align_up::<ALIGN>(Self::BYTES) };
            let slice = get_aligned_slice::<
                B,
                { ALIGN },
            >(buf, aligned_offset, word_size);
            let bytes = if is_big_endian::<B>() {
                self.to_be_bytes_vec()
            } else {
                self.to_le_bytes_vec()
            };
            if SOL_MODE {
                slice[word_size - Self::BYTES..].copy_from_slice(&bytes);
                slice[..word_size - Self::BYTES].fill(0);
            } else {
                slice[..Self::BYTES].copy_from_slice(&bytes);
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = if SOL_MODE {
                align_up::<32>(offset)
            } else {
                align_up::<ALIGN>(offset)
            };
            let word_size = if SOL_MODE { 32 } else { align_up::<ALIGN>(Self::BYTES) };
            if buf.remaining() < aligned_offset + word_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + word_size,
                        found: buf.remaining(),
                        msg: "buf too small to read Uint".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..aligned_offset + word_size];
            let value_slice = if SOL_MODE {
                &chunk[word_size - Self::BYTES..]
            } else {
                &chunk[..Self::BYTES]
            };
            let value = if is_big_endian::<B>() {
                Self::from_be_slice(value_slice)
            } else {
                Self::from_le_slice(value_slice)
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = if SOL_MODE {
                align_up::<32>(offset)
            } else {
                align_up::<ALIGN>(offset)
            };
            let word_size = if SOL_MODE { 32 } else { align_up::<ALIGN>(Self::BYTES) };
            Ok((aligned_offset, word_size))
        }
    }
}
pub mod hash {
    extern crate alloc;
    use crate::{
        bytes::{read_bytes_header, write_bytes, write_bytes_solidity, write_bytes_wasm},
        encoder::{align_up, read_u32_aligned, write_u32_aligned, Encoder},
        error::{CodecError, DecodingError},
    };
    use alloc::vec::Vec;
    use byteorder::ByteOrder;
    use bytes::{Buf, BytesMut};
    use core::{fmt::Debug, hash::Hash};
    use hashbrown::{HashMap, HashSet};
    /// Implement encoding for HashMap, SOL_MODE = false
    impl<K, V, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false>
    for HashMap<K, V>
    where
        K: Default + Sized + Encoder<B, { ALIGN }, false> + Eq + Hash + Ord,
        V: Default + Sized + Encoder<B, { ALIGN }, false>,
    {
        const HEADER_SIZE: usize = 4 + 8 + 8;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_el_size = align_up::<ALIGN>(4);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.len() < aligned_offset + aligned_header_size {
                buf.resize(aligned_offset + aligned_header_size, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, self.len() as u32);
            let mut entries: Vec<_> = self.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut key_buf = BytesMut::zeroed(
                align_up::<ALIGN>(K::HEADER_SIZE) * self.len(),
            );
            for (i, (key, _)) in entries.iter().enumerate() {
                let key_offset = align_up::<ALIGN>(K::HEADER_SIZE) * i;
                key.encode(&mut key_buf, key_offset)?;
            }
            write_bytes::<
                B,
                ALIGN,
                false,
            >(
                buf,
                aligned_offset + aligned_header_el_size,
                &key_buf,
                entries.len() as u32,
            );
            let mut value_buf = BytesMut::zeroed(
                align_up::<ALIGN>(V::HEADER_SIZE) * self.len(),
            );
            for (i, (_, value)) in entries.iter().enumerate() {
                let value_offset = align_up::<ALIGN>(V::HEADER_SIZE) * i;
                value.encode(&mut value_buf, value_offset)?;
            }
            write_bytes_wasm::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_header_el_size * 3, &value_buf);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<{ ALIGN }>(offset);
            let aligned_header_el_size = align_up::<ALIGN>(4);
            let aligned_header_size = align_up::<{ ALIGN }>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashMap header".to_string(),
                    }),
                );
            }
            let length = read_u32_aligned::<B, { ALIGN }>(buf, aligned_offset)? as usize;
            let (keys_offset, keys_length) = read_bytes_header::<
                B,
                { ALIGN },
                false,
            >(buf, aligned_offset + aligned_header_el_size)
                .unwrap();
            let (values_offset, values_length) = read_bytes_header::<
                B,
                { ALIGN },
                false,
            >(buf, aligned_offset + aligned_header_el_size * 3)
                .unwrap();
            {
                ::std::io::_print(
                    format_args!(
                        "values_offset: {0}, values_length: {1}\n",
                        values_offset,
                        values_length,
                    ),
                );
            };
            let key_bytes = &buf.chunk()[keys_offset..keys_offset + keys_length];
            let value_bytes = &buf.chunk()[values_offset..values_offset + values_length];
            let keys = (0..length)
                .map(|i| {
                    let key_offset = align_up::<{ ALIGN }>(K::HEADER_SIZE) * i;
                    K::decode(&key_bytes, key_offset).unwrap_or_default()
                });
            let values = (0..length)
                .map(|i| {
                    let value_offset = align_up::<{ ALIGN }>(V::HEADER_SIZE) * i;
                    V::decode(&value_bytes, value_offset).unwrap_or_default()
                });
            let result: HashMap<K, V> = keys.zip(values).collect();
            if result.len() != length {
                return Err(
                    CodecError::Decoding(
                        DecodingError::InvalidData({
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "Expected {0} elements, but decoded {1}",
                                    length,
                                    result.len(),
                                ),
                            );
                            res
                        }),
                    ),
                );
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashMap header".to_string(),
                    }),
                );
            }
            let (keys_offset, keys_length) = read_bytes_header::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset + align_up::<ALIGN>(4))
                .unwrap();
            let (_values_offset, values_length) = read_bytes_header::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset + align_up::<ALIGN>(12))
                .unwrap();
            Ok((keys_offset, keys_length + values_length))
        }
    }
    /// Implement encoding for HashMap, SOL_MODE = true
    impl<K, V, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true>
    for HashMap<K, V>
    where
        K: Debug + Default + Sized + Encoder<B, { ALIGN }, true> + Eq + Hash + Ord,
        V: Debug + Default + Sized + Encoder<B, { ALIGN }, true>,
    {
        const HEADER_SIZE: usize = 32 + 32 + 32 + 32;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            if buf.len() < aligned_offset + Self::HEADER_SIZE {
                buf.resize(aligned_offset + Self::HEADER_SIZE, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, (32) as u32);
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset + 32, self.len() as u32);
            let mut entries: Vec<_> = self.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut key_buf = BytesMut::zeroed(
                align_up::<ALIGN>(K::HEADER_SIZE) * self.len(),
            );
            for (i, (key, _)) in entries.iter().enumerate() {
                let key_offset = align_up::<ALIGN>(K::HEADER_SIZE) * i;
                key.encode(&mut key_buf, key_offset)?;
            }
            let relative_key_offset = buf.len() - aligned_offset - 64;
            write_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + 64, relative_key_offset as u32);
            write_bytes_solidity::<
                B,
                ALIGN,
            >(buf, aligned_offset + 64, &key_buf, entries.len() as u32);
            let relative_value_offset = buf.len() - aligned_offset - 96;
            write_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + 96, relative_value_offset as u32);
            let mut value_buf = BytesMut::zeroed(
                align_up::<ALIGN>(V::HEADER_SIZE) * self.len(),
            );
            for (i, (_, value)) in entries.iter().enumerate() {
                let value_offset = align_up::<ALIGN>(V::HEADER_SIZE) * i;
                value.encode(&mut value_buf, value_offset)?;
            }
            write_bytes_solidity::<
                B,
                ALIGN,
            >(buf, buf.len(), &value_buf, entries.len() as u32);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            const KEYS_OFFSET: usize = 32;
            const VALUES_OFFSET: usize = 64;
            let aligned_offset = align_up::<{ ALIGN }>(offset);
            let header_end = aligned_offset
                .checked_add(Self::HEADER_SIZE)
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            if buf.remaining()
                < usize::try_from(header_end)
                    .map_err(|_| CodecError::Decoding(DecodingError::Overflow))?
            {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: header_end as usize,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashMap header".to_string(),
                    }),
                );
            }
            let data_offset = read_u32_aligned::<B, { ALIGN }>(buf, aligned_offset)?
                as usize;
            let start_offset = aligned_offset
                .checked_add(data_offset)
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            let length = read_u32_aligned::<B, { ALIGN }>(buf, start_offset)? as usize;
            if length == 0 {
                return Ok(HashMap::new());
            }
            let keys_offset = read_u32_aligned::<
                B,
                { ALIGN },
            >(buf, start_offset + KEYS_OFFSET)? as usize;
            let values_offset = read_u32_aligned::<
                B,
                { ALIGN },
            >(buf, start_offset + VALUES_OFFSET)? as usize;
            let keys_start = keys_offset
                .checked_add(start_offset)
                .and_then(|sum| sum.checked_add(KEYS_OFFSET))
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            let values_start = values_offset
                .checked_add(start_offset)
                .and_then(|sum| sum.checked_add(VALUES_OFFSET))
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            let mut result = HashMap::with_capacity(length);
            let keys_data = &buf.chunk()[keys_start + 32..];
            let values_data = &buf.chunk()[values_start + 32..];
            for i in 0..length {
                let key_offset = align_up::<{ ALIGN }>(K::HEADER_SIZE)
                    .checked_mul(i)
                    .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
                let value_offset = align_up::<{ ALIGN }>(V::HEADER_SIZE)
                    .checked_mul(i)
                    .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
                let key = K::decode(&keys_data, key_offset)?;
                let value = V::decode(&values_data, value_offset)?;
                result.insert(key, value);
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashMap header".to_string(),
                    }),
                );
            }
            let (keys_offset, keys_length) = read_bytes_header::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset + align_up::<ALIGN>(4))
                .unwrap();
            let (_values_offset, values_length) = read_bytes_header::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset + align_up::<ALIGN>(12))
                .unwrap();
            Ok((keys_offset, keys_length + values_length))
        }
    }
    /// Implement encoding for HashSet, SOL_MODE = false
    impl<T, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for HashSet<T>
    where
        T: Default + Sized + Encoder<B, { ALIGN }, false> + Eq + Hash + Ord,
    {
        const HEADER_SIZE: usize = 4 + 8;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_el_size = align_up::<ALIGN>(4);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.len() < aligned_offset + aligned_header_size {
                buf.resize(aligned_offset + aligned_header_size, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, self.len() as u32);
            let mut entries: Vec<_> = self.iter().collect();
            entries.sort();
            let mut value_buf = BytesMut::zeroed(
                align_up::<ALIGN>(T::HEADER_SIZE) * self.len(),
            );
            for (i, value) in entries.iter().enumerate() {
                let value_offset = align_up::<ALIGN>(T::HEADER_SIZE) * i;
                value.encode(&mut value_buf, value_offset)?;
            }
            write_bytes::<
                B,
                ALIGN,
                false,
            >(
                buf,
                aligned_offset + aligned_header_el_size,
                &value_buf,
                entries.len() as u32,
            );
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashSet header".to_string(),
                    }),
                );
            }
            let length = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
            let (data_offset, data_length) = read_bytes_header::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset + align_up::<ALIGN>(4))?;
            let mut result = HashSet::with_capacity(length);
            let value_bytes = &buf.chunk()[data_offset..data_offset + data_length];
            for i in 0..length {
                let value_offset = align_up::<ALIGN>(T::HEADER_SIZE) * i;
                let value = T::decode(&value_bytes, value_offset)?;
                result.insert(value);
            }
            if result.len() != length {
                return Err(
                    CodecError::Decoding(
                        DecodingError::InvalidData({
                            let res = ::alloc::fmt::format(
                                format_args!(
                                    "Expected {0} elements, but decoded {1}",
                                    length,
                                    result.len(),
                                ),
                            );
                            res
                        }),
                    ),
                );
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashSet header".to_string(),
                    }),
                );
            }
            let (data_offset, data_length) = read_bytes_header::<
                B,
                ALIGN,
                false,
            >(buf, aligned_offset + align_up::<ALIGN>(4))?;
            Ok((data_offset, data_length))
        }
    }
    /// Implement encoding for HashSet, SOL_MODE = true
    impl<T, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for HashSet<T>
    where
        T: Debug + Default + Sized + Encoder<B, { ALIGN }, true> + Eq + Hash + Ord,
    {
        const HEADER_SIZE: usize = 32 + 32 + 32;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            if buf.len() < aligned_offset + Self::HEADER_SIZE {
                buf.resize(aligned_offset + Self::HEADER_SIZE, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, 32 as u32);
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset + 32, self.len() as u32);
            let mut entries: Vec<_> = self.iter().collect();
            entries.sort();
            let mut value_buf = BytesMut::zeroed(
                align_up::<ALIGN>(T::HEADER_SIZE) * self.len(),
            );
            for (i, value) in entries.iter().enumerate() {
                let value_offset = align_up::<ALIGN>(T::HEADER_SIZE) * i;
                value.encode(&mut value_buf, value_offset)?;
            }
            let relative_data_offset = buf.len() - aligned_offset - 64;
            write_u32_aligned::<
                B,
                ALIGN,
            >(buf, aligned_offset + 64, relative_data_offset as u32);
            write_bytes_solidity::<
                B,
                ALIGN,
            >(buf, buf.len(), &value_buf, entries.len() as u32);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            const DATA_OFFSET: usize = 32;
            let aligned_offset = align_up::<{ ALIGN }>(offset);
            let header_end = aligned_offset
                .checked_add(Self::HEADER_SIZE)
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            if buf.remaining()
                < usize::try_from(header_end)
                    .map_err(|_| CodecError::Decoding(DecodingError::Overflow))?
            {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: header_end as usize,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashSet header".to_string(),
                    }),
                );
            }
            let data_offset = read_u32_aligned::<B, { ALIGN }>(buf, aligned_offset)?
                as usize;
            let start_offset = aligned_offset
                .checked_add(data_offset)
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            let length = read_u32_aligned::<B, { ALIGN }>(buf, start_offset)? as usize;
            if length == 0 {
                return Ok(HashSet::new());
            }
            {
                ::std::io::_print(format_args!("length: {0}\n", length));
            };
            let values_offset = read_u32_aligned::<
                B,
                { ALIGN },
            >(buf, start_offset + DATA_OFFSET)? as usize;
            {
                ::std::io::_print(format_args!("values_offset: {0}\n", values_offset));
            };
            let values_start = values_offset
                .checked_add(start_offset)
                .and_then(|sum| sum.checked_add(DATA_OFFSET))
                .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
            {
                ::std::io::_print(format_args!("values_start: {0}\n", values_start));
            };
            let mut result = HashSet::with_capacity(length);
            let values_data = &buf.chunk()[values_start + 32..];
            for i in 0..length {
                let value_offset = align_up::<{ ALIGN }>(T::HEADER_SIZE)
                    .checked_mul(i)
                    .ok_or_else(|| CodecError::Decoding(DecodingError::Overflow))?;
                let value = T::decode(&values_data, value_offset)?;
                result.insert(value);
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "Not enough data to decode HashSet header".to_string(),
                    }),
                );
            }
            let data_offset = read_u32_aligned::<B, { ALIGN }>(buf, aligned_offset)?
                as usize;
            let start_offset = aligned_offset + data_offset;
            let length = read_u32_aligned::<B, { ALIGN }>(buf, start_offset)? as usize;
            let values_offset = read_u32_aligned::<B, { ALIGN }>(buf, start_offset + 64)?
                as usize;
            let values_start = start_offset + 64 + values_offset;
            let data_length = length * align_up::<{ ALIGN }>(T::HEADER_SIZE);
            Ok((values_start + 32, data_length))
        }
    }
}
pub mod primitive {
    extern crate alloc;
    use crate::{
        encoder::{
            align_up, get_aligned_indices, get_aligned_slice, is_big_endian, Encoder,
        },
        error::{CodecError, DecodingError},
    };
    use byteorder::ByteOrder;
    use bytes::{Buf, BytesMut};
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for u8 {
        const HEADER_SIZE: usize = core::mem::size_of::<u8>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let write_to = get_aligned_slice::<B, ALIGN>(buf, aligned_offset, 1);
            write_to[0] = *self;
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + word_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + word_size,
                        found: buf.remaining(),
                        msg: "buf too small to read aligned u8".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                chunk[word_size - 1]
            } else {
                chunk[0]
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            _offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((0, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for bool {
        const HEADER_SIZE: usize = core::mem::size_of::<bool>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let value: u8 = if *self { 1 } else { 0 };
            <u8 as Encoder<B, { ALIGN }, { SOL_MODE }>>::encode(&value, buf, offset)
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let value = <u8 as Encoder<
                B,
                { ALIGN },
                { SOL_MODE },
            >>::decode(buf, offset)?;
            Ok(value != 0)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for u16 {
        const HEADER_SIZE: usize = core::mem::size_of::<u16>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let (start, end) = get_aligned_indices::<
                B,
                ALIGN,
            >(aligned_offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE);
            B::write_u16(&mut buf[start..end], *self);
            let fill_val = if *self > 0 { 0x00 } else { 0xFF };
            {
                ::std::io::_print(format_args!("start: {0}, end: {1}\n", start, end));
            };
            {
                ::std::io::_print(format_args!("fill_val: {0}\n", fill_val));
            };
            for i in aligned_offset..start {
                buf[i] = fill_val;
            }
            B::write_u16(&mut buf[start..end], *self);
            for i in end..(aligned_offset + word_size) {
                buf[i] = fill_val;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + ALIGN {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                B::read_u16(
                    &chunk[word_size
                        - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE..word_size],
                )
            } else {
                B::read_u16(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for u32 {
        const HEADER_SIZE: usize = core::mem::size_of::<u32>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let (start, end) = get_aligned_indices::<
                B,
                ALIGN,
            >(aligned_offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE);
            B::write_u32(&mut buf[start..end], *self);
            let fill_val = if *self > 0 { 0x00 } else { 0xFF };
            {
                ::std::io::_print(format_args!("start: {0}, end: {1}\n", start, end));
            };
            {
                ::std::io::_print(format_args!("fill_val: {0}\n", fill_val));
            };
            for i in aligned_offset..start {
                buf[i] = fill_val;
            }
            B::write_u32(&mut buf[start..end], *self);
            for i in end..(aligned_offset + word_size) {
                buf[i] = fill_val;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + ALIGN {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                B::read_u32(
                    &chunk[word_size
                        - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE..word_size],
                )
            } else {
                B::read_u32(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for u64 {
        const HEADER_SIZE: usize = core::mem::size_of::<u64>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let (start, end) = get_aligned_indices::<
                B,
                ALIGN,
            >(aligned_offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE);
            B::write_u64(&mut buf[start..end], *self);
            let fill_val = if *self > 0 { 0x00 } else { 0xFF };
            {
                ::std::io::_print(format_args!("start: {0}, end: {1}\n", start, end));
            };
            {
                ::std::io::_print(format_args!("fill_val: {0}\n", fill_val));
            };
            for i in aligned_offset..start {
                buf[i] = fill_val;
            }
            B::write_u64(&mut buf[start..end], *self);
            for i in end..(aligned_offset + word_size) {
                buf[i] = fill_val;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + ALIGN {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                B::read_u64(
                    &chunk[word_size
                        - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE..word_size],
                )
            } else {
                B::read_u64(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for i16 {
        const HEADER_SIZE: usize = core::mem::size_of::<i16>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let (start, end) = get_aligned_indices::<
                B,
                ALIGN,
            >(aligned_offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE);
            B::write_i16(&mut buf[start..end], *self);
            let fill_val = if *self > 0 { 0x00 } else { 0xFF };
            {
                ::std::io::_print(format_args!("start: {0}, end: {1}\n", start, end));
            };
            {
                ::std::io::_print(format_args!("fill_val: {0}\n", fill_val));
            };
            for i in aligned_offset..start {
                buf[i] = fill_val;
            }
            B::write_i16(&mut buf[start..end], *self);
            for i in end..(aligned_offset + word_size) {
                buf[i] = fill_val;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + ALIGN {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                B::read_i16(
                    &chunk[word_size
                        - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE..word_size],
                )
            } else {
                B::read_i16(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for i32 {
        const HEADER_SIZE: usize = core::mem::size_of::<i32>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let (start, end) = get_aligned_indices::<
                B,
                ALIGN,
            >(aligned_offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE);
            B::write_i32(&mut buf[start..end], *self);
            let fill_val = if *self > 0 { 0x00 } else { 0xFF };
            {
                ::std::io::_print(format_args!("start: {0}, end: {1}\n", start, end));
            };
            {
                ::std::io::_print(format_args!("fill_val: {0}\n", fill_val));
            };
            for i in aligned_offset..start {
                buf[i] = fill_val;
            }
            B::write_i32(&mut buf[start..end], *self);
            for i in end..(aligned_offset + word_size) {
                buf[i] = fill_val;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + ALIGN {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                B::read_i32(
                    &chunk[word_size
                        - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE..word_size],
                )
            } else {
                B::read_i32(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, ALIGN, SOL_MODE> for i64 {
        const HEADER_SIZE: usize = core::mem::size_of::<i64>();
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.len() < aligned_offset + word_size {
                buf.resize(aligned_offset + word_size, 0);
            }
            let (start, end) = get_aligned_indices::<
                B,
                ALIGN,
            >(aligned_offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE);
            B::write_i64(&mut buf[start..end], *self);
            let fill_val = if *self > 0 { 0x00 } else { 0xFF };
            {
                ::std::io::_print(format_args!("start: {0}, end: {1}\n", start, end));
            };
            {
                ::std::io::_print(format_args!("fill_val: {0}\n", fill_val));
            };
            for i in aligned_offset..start {
                buf[i] = fill_val;
            }
            B::write_i64(&mut buf[start..end], *self);
            for i in end..(aligned_offset + word_size) {
                buf[i] = fill_val;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let word_size = align_up::<
                ALIGN,
            >(ALIGN.max(<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE));
            if buf.remaining() < aligned_offset + ALIGN {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + ALIGN,
                        found: buf.remaining(),
                        msg: "buf too small to decode value".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let value = if is_big_endian::<B>() {
                B::read_i64(
                    &chunk[word_size
                        - <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE..word_size],
                )
            } else {
                B::read_i64(&chunk[..<Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE])
            };
            Ok(value)
        }
        fn partial_decode(
            _buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            Ok((offset, <Self as Encoder<B, ALIGN, SOL_MODE>>::HEADER_SIZE))
        }
    }
    /// Encodes and decodes Option<T> where T is an Encoder.
    /// The encoded data is prefixed with a single byte that indicates whether the Option is Some or
    /// None. Single byte will be aligned to ALIGN.
    impl<
        T,
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for Option<T>
    where
        T: Sized + Encoder<B, { ALIGN }, { SOL_MODE }> + Default,
    {
        const HEADER_SIZE: usize = 1 + T::HEADER_SIZE;
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let required_space = aligned_offset + ALIGN.max(Self::HEADER_SIZE);
            if buf.len() < required_space {
                buf.resize(required_space, 0);
            }
            let flag_slice = get_aligned_slice::<B, ALIGN>(buf, aligned_offset, 1);
            flag_slice[0] = if self.is_some() { 1 } else { 0 };
            let inner_offset = aligned_offset + ALIGN;
            match self {
                Some(inner_value) => inner_value.encode(buf, inner_offset)?,
                None => {
                    let default_value = T::default();
                    default_value.encode(buf, inner_offset)?;
                }
            };
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_data_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_data_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_data_size,
                        found: buf.remaining(),
                        msg: "buf too small".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let option_flag = if is_big_endian::<B>() {
                chunk[aligned_data_size - 1]
            } else {
                chunk[0]
            };
            let chunk = &buf.chunk()[aligned_offset + ALIGN..];
            if option_flag != 0 {
                let inner_value = T::decode(&chunk, 0)?;
                Ok(Some(inner_value))
            } else {
                Ok(None)
            }
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_size = align_up::<ALIGN>(Self::HEADER_SIZE);
            if buf.remaining() < aligned_offset + aligned_header_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_size,
                        found: buf.remaining(),
                        msg: "buf too small".to_string(),
                    }),
                );
            }
            let chunk = &buf.chunk()[aligned_offset..];
            let option_flag = if is_big_endian::<B>() {
                chunk[ALIGN - 1]
            } else {
                chunk[0]
            };
            let chunk = &buf.chunk()[aligned_offset + ALIGN..];
            if option_flag != 0 {
                let (_, inner_size) = T::partial_decode(&chunk, 0)?;
                Ok((aligned_offset, aligned_header_size + inner_size))
            } else {
                let aligned_data_size = align_up::<ALIGN>(T::HEADER_SIZE);
                Ok((aligned_offset, aligned_header_size + aligned_data_size))
            }
        }
    }
    impl<
        T,
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        const N: usize,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for [T; N]
    where
        T: Sized + Encoder<B, { ALIGN }, { SOL_MODE }> + Default + Copy,
    {
        const HEADER_SIZE: usize = T::HEADER_SIZE * N;
        const IS_DYNAMIC: bool = false;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let item_size = align_up::<ALIGN>(T::HEADER_SIZE);
            let total_size = aligned_offset + item_size * N;
            if buf.len() < total_size {
                buf.resize(total_size, 0);
            }
            for (i, item) in self.iter().enumerate() {
                item.encode(buf, aligned_offset + i * item_size)?;
            }
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let item_size = align_up::<ALIGN>(T::HEADER_SIZE);
            let total_size = aligned_offset + item_size * N;
            if buf.remaining() < total_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: total_size,
                        found: buf.remaining(),
                        msg: "buf too small".to_string(),
                    }),
                );
            }
            let mut result = [T::default(); N];
            let elem_size = align_up::<ALIGN>(T::HEADER_SIZE + T::HEADER_SIZE);
            for (i, item) in result.iter_mut().enumerate() {
                *item = T::decode(buf, aligned_offset + i * elem_size)?;
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let item_size = align_up::<ALIGN>(T::HEADER_SIZE);
            let total_size = item_size * N;
            if buf.remaining() < aligned_offset + total_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + total_size,
                        found: buf.remaining(),
                        msg: "Buffer too small to decode array".to_string(),
                    }),
                );
            }
            Ok((aligned_offset, total_size))
        }
    }
}
pub mod tuple {
    use crate::{
        encoder::{align_up, Encoder},
        error::CodecError,
    };
    use byteorder::ByteOrder;
    use bytes::{Buf, BytesMut};
    impl<
        T,
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T,)
    where
        T: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = align_up::<ALIGN>(T::HEADER_SIZE);
        const IS_DYNAMIC: bool = T::IS_DYNAMIC;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            self.0.encode(buf, offset)
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            Ok((T::decode(buf, offset)?,))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            T::partial_decode(buf, offset)
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
        T3,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2, T3)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T3: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T3::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic |= T3::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            self.2.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
                {
                    let value = T3::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T3::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
        T3,
        T4,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2, T3, T4)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T3: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T4: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T3::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T4::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic |= T3::IS_DYNAMIC;
            is_dynamic |= T4::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            self.2.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
            self.3.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
                {
                    let value = T3::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
                    value
                },
                {
                    let value = T4::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T3::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T4::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
        T3,
        T4,
        T5,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2, T3, T4, T5)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T3: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T4: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T5: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T3::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T4::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T5::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic |= T3::IS_DYNAMIC;
            is_dynamic |= T4::IS_DYNAMIC;
            is_dynamic |= T5::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            self.2.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
            self.3.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
            self.4.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
                {
                    let value = T3::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
                    value
                },
                {
                    let value = T4::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
                    value
                },
                {
                    let value = T5::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T3::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T4::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T5::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2, T3, T4, T5, T6)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T3: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T4: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T5: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T6: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T3::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T4::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T5::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T6::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic |= T3::IS_DYNAMIC;
            is_dynamic |= T4::IS_DYNAMIC;
            is_dynamic |= T5::IS_DYNAMIC;
            is_dynamic |= T6::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            self.2.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
            self.3.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
            self.4.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
            self.5.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T6::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
                {
                    let value = T3::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
                    value
                },
                {
                    let value = T4::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
                    value
                },
                {
                    let value = T5::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
                    value
                },
                {
                    let value = T6::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T6::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T3::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T4::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T5::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T6::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2, T3, T4, T5, T6, T7)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T3: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T4: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T5: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T6: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T7: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T3::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T4::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T5::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T6::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T7::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic |= T3::IS_DYNAMIC;
            is_dynamic |= T4::IS_DYNAMIC;
            is_dynamic |= T5::IS_DYNAMIC;
            is_dynamic |= T6::IS_DYNAMIC;
            is_dynamic |= T7::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            self.2.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
            self.3.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
            self.4.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
            self.5.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T6::HEADER_SIZE);
            self.6.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T7::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
                {
                    let value = T3::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
                    value
                },
                {
                    let value = T4::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
                    value
                },
                {
                    let value = T5::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
                    value
                },
                {
                    let value = T6::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T6::HEADER_SIZE);
                    value
                },
                {
                    let value = T7::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T7::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T3::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T4::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T5::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T6::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T7::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
    impl<
        B: ByteOrder,
        const ALIGN: usize,
        const SOL_MODE: bool,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
    > Encoder<B, { ALIGN }, { SOL_MODE }> for (T1, T2, T3, T4, T5, T6, T7, T8)
    where
        T1: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T2: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T3: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T4: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T5: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T6: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T7: Encoder<B, { ALIGN }, { SOL_MODE }>,
        T8: Encoder<B, { ALIGN }, { SOL_MODE }>,
    {
        const HEADER_SIZE: usize = {
            let mut size = 0;
            size = align_up::<ALIGN>(size);
            size += T1::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T2::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T3::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T4::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T5::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T6::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T7::HEADER_SIZE;
            size = align_up::<ALIGN>(size);
            size += T8::HEADER_SIZE;
            align_up::<ALIGN>(size)
        };
        const IS_DYNAMIC: bool = {
            let mut is_dynamic = false;
            is_dynamic |= T1::IS_DYNAMIC;
            is_dynamic |= T2::IS_DYNAMIC;
            is_dynamic |= T3::IS_DYNAMIC;
            is_dynamic |= T4::IS_DYNAMIC;
            is_dynamic |= T5::IS_DYNAMIC;
            is_dynamic |= T6::IS_DYNAMIC;
            is_dynamic |= T7::IS_DYNAMIC;
            is_dynamic |= T8::IS_DYNAMIC;
            is_dynamic
        };
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            self.0.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
            self.1.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
            self.2.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
            self.3.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
            self.4.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
            self.5.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T6::HEADER_SIZE);
            self.6.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T7::HEADER_SIZE);
            self.7.encode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + T8::HEADER_SIZE);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let mut current_offset = align_up::<ALIGN>(offset);
            Ok((
                {
                    let value = T1::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T1::HEADER_SIZE);
                    value
                },
                {
                    let value = T2::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T2::HEADER_SIZE);
                    value
                },
                {
                    let value = T3::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T3::HEADER_SIZE);
                    value
                },
                {
                    let value = T4::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T4::HEADER_SIZE);
                    value
                },
                {
                    let value = T5::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T5::HEADER_SIZE);
                    value
                },
                {
                    let value = T6::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T6::HEADER_SIZE);
                    value
                },
                {
                    let value = T7::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T7::HEADER_SIZE);
                    value
                },
                {
                    let value = T8::decode(buf, current_offset)?;
                    current_offset = align_up::<ALIGN>(current_offset + T8::HEADER_SIZE);
                    value
                },
            ))
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            let mut total_size = 0;
            let mut current_offset = align_up::<ALIGN>(offset);
            let (_, size) = T1::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T2::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T3::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T4::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T5::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T6::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T7::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            let (_, size) = T8::partial_decode(buf, current_offset)?;
            current_offset = align_up::<ALIGN>(current_offset + size);
            total_size += size;
            Ok((offset, total_size))
        }
    }
}
pub mod vec {
    extern crate alloc;
    use crate::{
        bytes::{
            read_bytes_header, read_bytes_wasm, write_bytes_solidity, write_bytes_wasm,
        },
        encoder::{align_up, read_u32_aligned, write_u32_aligned, Encoder},
        error::{CodecError, DecodingError},
    };
    use alloc::vec::Vec;
    use byteorder::ByteOrder;
    use bytes::{Buf, BytesMut};
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
    ///
    /// Implementation for non-Solidity mode
    impl<T, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, false> for Vec<T>
    where
        T: Default + Sized + Encoder<B, { ALIGN }, false> + std::fmt::Debug,
    {
        const HEADER_SIZE: usize = core::mem::size_of::<u32>() * 3;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_elem_size = align_up::<ALIGN>(4);
            let aligned_header_size = aligned_elem_size * 3;
            if buf.len() < aligned_offset + aligned_header_size {
                buf.resize(aligned_offset + aligned_header_size, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, self.len() as u32);
            if self.is_empty() {
                write_u32_aligned::<
                    B,
                    ALIGN,
                >(buf, aligned_offset + aligned_elem_size, aligned_header_size as u32);
                write_u32_aligned::<
                    B,
                    ALIGN,
                >(buf, aligned_offset + aligned_elem_size * 2, 0);
                return Ok(());
            }
            let mut value_encoder = BytesMut::zeroed(
                ALIGN.max(T::HEADER_SIZE) * self.len(),
            );
            for (index, obj) in self.iter().enumerate() {
                let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;
                obj.encode(&mut value_encoder, elem_offset)?;
            }
            let data = value_encoder.freeze();
            write_bytes_wasm::<B, ALIGN>(buf, aligned_offset + aligned_elem_size, &data);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let aligned_header_el_size = align_up::<ALIGN>(4);
            if buf.remaining() < aligned_offset + aligned_header_el_size {
                return Err(
                    CodecError::Decoding(DecodingError::BufferTooSmall {
                        expected: aligned_offset + aligned_header_el_size,
                        found: buf.remaining(),
                        msg: "failed to decode vector length".to_string(),
                    }),
                );
            }
            let data_len = read_u32_aligned::<B, ALIGN>(buf, aligned_offset)? as usize;
            if data_len == 0 {
                return Ok(Vec::new());
            }
            let mut result = Vec::with_capacity(data_len);
            let data = read_bytes_wasm::<
                B,
                ALIGN,
            >(buf, aligned_offset + aligned_header_el_size)?;
            for i in 0..data_len {
                let elem_offset = i * align_up::<ALIGN>(T::HEADER_SIZE);
                let value = T::decode(&data, elem_offset)?;
                result.push(value);
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            read_bytes_header::<B, ALIGN, false>(buf, offset)
        }
    }
    impl<T, B: ByteOrder, const ALIGN: usize> Encoder<B, { ALIGN }, true> for Vec<T>
    where
        T: Default + Sized + Encoder<B, { ALIGN }, true> + std::fmt::Debug,
    {
        const HEADER_SIZE: usize = 32;
        const IS_DYNAMIC: bool = true;
        fn encode(&self, buf: &mut BytesMut, offset: usize) -> Result<(), CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            if buf.len() < aligned_offset + 32 {
                buf.resize(aligned_offset + 32, 0);
            }
            write_u32_aligned::<B, ALIGN>(buf, aligned_offset, buf.len() as u32);
            if self.is_empty() {
                write_u32_aligned::<B, ALIGN>(buf, buf.len(), 0);
                return Ok(());
            }
            let mut value_encoder = BytesMut::zeroed(32 * self.len());
            for (index, obj) in self.iter().enumerate() {
                let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;
                obj.encode(&mut value_encoder, elem_offset)?;
            }
            let data = value_encoder.freeze();
            write_bytes_solidity::<
                B,
                ALIGN,
            >(buf, aligned_offset + 32, &data, self.len() as u32);
            Ok(())
        }
        fn decode(buf: &impl Buf, offset: usize) -> Result<Self, CodecError> {
            let aligned_offset = align_up::<ALIGN>(offset);
            let (data_offset, data_len) = Self::partial_decode(buf, aligned_offset)?;
            if data_len == 0 {
                return Ok(Vec::new());
            }
            let mut result = Vec::with_capacity(data_len);
            for i in 0..data_len {
                let elem_offset = i * align_up::<ALIGN>(T::HEADER_SIZE);
                let value = T::decode(&&buf.chunk()[data_offset..], elem_offset)?;
                result.push(value);
            }
            Ok(result)
        }
        fn partial_decode(
            buf: &impl Buf,
            offset: usize,
        ) -> Result<(usize, usize), CodecError> {
            read_bytes_header::<B, ALIGN, true>(buf, offset)
        }
    }
}
