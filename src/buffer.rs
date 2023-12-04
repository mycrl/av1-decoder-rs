pub struct Buffer<'a> {
    buf: &'a [u8],
    index: usize,
    bit_pos: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            index: 0,
            bit_pos: 0,
        }
    }

    pub fn seek_bits(&mut self, cut: usize) {
        for _ in 0..cut {
            self.advance();
        }
    }

    pub fn get_bytes(&mut self, count: usize) -> &[u8] {
        assert_eq!(self.bit_pos, 0);

        self.index += count;
        &self.buf[self.index - count..self.index]
    }

    pub fn get_bit(&mut self) -> bool {
        self.next()
    }

    /// Unsigned n-bit number appearing directly in the bitstream. The bits are
    /// read from high to low order.
    pub fn get_bits(&mut self, count: usize) -> u32 {
        assert!(count > 0 && count <= 32);

        let mut aac = 0;
        for i in 0..count {
            aac |= (self.get_bit() as u32) << (count - i - 1);
        }

        aac
    }

    /// Variable length unsigned n-bit number appearing directly in the
    /// bitstream.
    pub fn get_uvlc(&mut self) -> u32 {
        let mut lz = 0;
        loop {
            if self.get_bit() {
                break;
            }

            lz += 1;
        }

        if lz >= 32 {
            0xFFFFFFFF
        } else {
            self.get_bits(lz) + (1 << lz) - 1
        }
    }

    /// Unsigned little-endian n-byte number appearing directly in the
    /// bitstream.
    ///
    /// Note: This syntax element will only be present when the bitstream
    /// position is byte aligned.
    pub fn get_le(&mut self, count: usize) -> u32 {
        assert_eq!(self.bit_pos, 0);

        let mut t = 0;
        for i in 0..count {
            t += self.get_bits(8) << (i * 8);
        }

        t
    }

    /// Unsigned integer represented by a variable number of little-endian
    /// bytes.
    ///
    /// Note: This syntax element will only be present when the bitstream
    /// position is byte aligned.
    ///
    /// In this encoding, the most significant bit of each byte is equal to 1 to
    /// signal that more bytes should be read, or equal to 0 to signal the
    /// end of the encoding.
    ///
    /// A variable Leb128Bytes is set equal to the number of bytes read during
    /// this process.
    ///
    /// It is a requirement of bitstream conformance that the value returned
    /// from the leb128 parsing process is less than or equal to (1 << 32) -
    /// 1.
    ///
    /// leb128_byte contains 8 bits read from the bitstream. The bottom 7 bits
    /// are used to compute the variable value. The most significant bit is
    /// used to indicate that there are more bytes to be read.
    ///
    /// It is a requirement of bitstream conformance that the most significant
    /// bit of leb128_byte is equal to 0 if i is equal to 7. (This
    /// ensures that this syntax descriptor never uses more than 8 bytes.)
    pub fn get_leb128(&mut self) -> u16 {
        assert_eq!(self.bit_pos, 0);

        let mut value = 0;
        for i in 0..8 {
            let byte = self.get_bits(8) as u16;
            value |= (byte & 0x7f) << (i * 7);
            if byte & 0x80 == 0 {
                break;
            }
        }

        value
    }

    /// Signed integer converted from an n bits unsigned integer in the
    /// bitstream. (The unsigned integer corresponds to the bottom n bits of
    /// the signed integer.)
    pub fn get_su(&mut self, count: usize) -> i32 {
        let mut value = self.get_bits(count) as i32;
        let sign_mask = 1 << (count - 1) as i32;

        if value > 0 && sign_mask > 0 {
            value = value - 2 * sign_mask;
        }

        value
    }
}

impl<'a> Buffer<'a> {
    fn advance(&mut self) {
        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            if self.index < self.buf.len() {
                self.index += 1;
            }
        }
    }

    fn next(&mut self) -> bool {
        let curr_byte = self.buf[self.index];
        let shift = 7 - self.bit_pos;
        let bit = curr_byte & (1 << shift);

        self.advance();
        (bit >> shift) == 1
    }
}

impl<'a> AsMut<Buffer<'a>> for Buffer<'a> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
