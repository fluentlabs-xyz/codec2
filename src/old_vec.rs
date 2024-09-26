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

        // Check if we can store header
        if buf.len() < aligned_offset + aligned_header_size {
            buf.resize(aligned_offset + aligned_header_size, 0);
        }

        if SOLIDITY_COMP {
            // Solidity mode: write offset only (current buffer length)
            write_u32_aligned::<B, ALIGN, true>(buf, aligned_offset, buf.len() as u32);
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

        let header_size = if SOLIDITY_COMP { 4 } else { T::HEADER_SIZE };
        // Encode values
        let mut value_encoder = BytesMut::zeroed(align_up::<ALIGN>(header_size) * self.len());

        for (index, obj) in self.iter().enumerate() {
            let elem_offset = ALIGN.max(T::HEADER_SIZE) * index;

            obj.encode::<B, ALIGN, SOLIDITY_COMP>(&mut value_encoder, elem_offset)
                .expect("Failed to encode vector element");
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

        // if SOLIDITY_COMP {
        //     return decode_vec_solidity_nested2::<B, T, ALIGN>(buf, aligned_offset);

        // return decode_vec_solidity::<B, T, ALIGN>(buf, offset);
        // }

        let (data_offset, number_of_elements) =
            Self::partial_decode::<B, ALIGN, SOLIDITY_COMP>(buf, aligned_offset)?;

        println!(")()(Data offset: {:?}", data_offset);
        println!(")()(Data bytes len: {:?}", number_of_elements);
        let data_len = if SOLIDITY_COMP {
            number_of_elements
        } else {
            read_u32_aligned::<B, ALIGN, false>(buf, aligned_offset)? as usize
        };

        if data_len == 0 {
            return Ok(Vec::new());
        }

        // let header_size = if SOLIDITY_COMP { 8 } else { Self::HEADER_SIZE };

        println!("Data offset: {:?}", data_offset);
        println!("Buf {:?}", &buf.chunk()[..]);

        // let mut input_bytes = read_bytes::<B, ALIGN, SOLIDITY_COMP>(
        //     buf,
        //     data_offset,
        //     val_size,
        // )?;
        // let real_values = input_bytes.to_vec();
        // println!("Real values: {:?}", real_values);
        let mut result = Vec::with_capacity(number_of_elements);
        // println!("Input bytes len: {:?}", input_bytes.len());
        // println!("input bytes: {:?}", input_bytes.to_vec());

        // let mut input_bytes = input_bytes.clone();
        for i in 0..data_len {
            let elem_offset = i * align_up::<ALIGN>(T::HEADER_SIZE);

            let input_bytes = Bytes::copy_from_slice(&buf.chunk()[data_offset..]);
            let value = T::decode::<B, ALIGN, SOLIDITY_COMP>(&input_bytes, elem_offset)?;

            result.push(value);
        }

        Ok(result)
    }

    /// Partial decode is used to get the offset and length of the vector without decoding the whole vector.
    fn partial_decode<B: ByteOrder, const ALIGN: usize, const SOLIDITY_COMP: bool>(
        buf: &(impl Buf + ?Sized),
        offset: usize,
    ) -> Result<(usize, usize), CodecError> {
        read_bytes_header::<B, ALIGN, SOLIDITY_COMP>(buf, offset)
    }
}
