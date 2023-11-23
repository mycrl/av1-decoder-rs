/// An Exponential-Golomb parser.
pub struct ExpGolombDecoder<'a> {
    iter: BitIterator<'a>,
}

impl<'a> ExpGolombDecoder<'a> {
    #[inline]
    #[must_use]
    pub fn new(buf: &'a [u8], start: u32) -> Self {
        Self {
            iter: BitIterator::new(buf, start),
        }
    }

    #[inline]
    pub fn get_bits(&mut self, count: usize) -> u32 {
        let mut ret = 0;
        for i in 0..count {
            ret |= (self.get_bit() as u32) << (count - i - 1);
        }

        ret
    }

    #[inline]
    pub fn get_bit(&mut self) -> bool {
        self.iter.next().expect("have reached the end!")
    }

    #[inline]
    fn count_leading_zeroes(&mut self) -> Option<u32> {
        let mut leading_zeros = 0;
        for bit in self.iter.by_ref() {
            if !bit {
                leading_zeros += 1;
                if leading_zeros > u64::BITS {
                    return None;
                }
            } else {
                return Some(leading_zeros);
            }
        }

        None
    }

    #[inline]
    pub fn get_unsigned(&mut self) -> usize {
        let mut lz = self.count_leading_zeroes().expect("have reached the end!");
        let x = (1u64.wrapping_shl(lz) - 1) as usize;
        let mut y = 0;

        if lz != 0 {
            for bit in self.iter.by_ref() {
                y <<= 1;
                y |= bit as usize;
                lz -= 1;
                if lz == 0 {
                    break;
                }
            }

            if lz != 0 {
                panic!("have reached the end!")
            }
        }

        x + y
    }

    #[inline]
    pub fn get_signed(&mut self) -> isize {
        let k = self.get_unsigned();
        let factor = if k % 2 == 0 { -1 } else { 1 };
        factor * (k / 2 + k % 2) as isize
    }

    #[inline]
    pub fn skip_next(&mut self) {
        if let Some(lz) = self.count_leading_zeroes() {
            self.iter.skip_bits(lz);
        }
    }
}

struct BitIterator<'a> {
    buf: &'a [u8],
    index: usize,
    bit_pos: u32,
}

impl<'a> BitIterator<'a> {
    #[inline]
    fn new(buf: &'a [u8], shift_sub: u32) -> BitIterator<'a> {
        Self {
            buf,
            index: 0,
            bit_pos: shift_sub,
        }
    }

    #[inline]
    fn skip_bits(&mut self, num_bits: u32) {
        let offset = self.bit_pos as usize + num_bits as usize;
        self.index = usize::min(self.buf.len(), self.index + offset / 8);
        self.bit_pos = (offset % 8) as u32;
    }
}

impl<'a> core::iter::Iterator for BitIterator<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let curr_byte = *self.buf.get(self.index)?;
        let shift = 7 - self.bit_pos;
        let bit = curr_byte & (1 << shift);

        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            if self.index < self.buf.len() {
                self.index += 1;
            }
        }

        Some((bit >> shift) == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifted_data() {
        let data: [(&[u8], u32, usize); 8] = [
            (&[0b01000000], 0, 1),
            (&[0b00100000], 1, 1),
            (&[0b00010000], 2, 1),
            (&[0b00001000], 3, 1),
            (&[0b00000100], 4, 1),
            (&[0b00000010], 5, 1),
            (&[0b00000001, 0], 6, 1),
            (&[0b00000000, 0b10000000], 7, 1),
        ];

        for (buf, start, ans) in data {
            let mut reader = ExpGolombDecoder::new(buf, start);
            let res = reader.get_unsigned();
            assert_eq!(res, ans);
        }
    }

    #[test]
    fn mix_next_unsigned_with_next_bit() {
        let data = [0b01010101];
        let mut reader = ExpGolombDecoder::new(&data, 0);
        assert_eq!(reader.get_unsigned(), 1);
        assert_eq!(reader.get_bit(), true);
        assert_eq!(reader.get_unsigned(), 1);
        assert_eq!(reader.get_bit(), true);
    }
}
