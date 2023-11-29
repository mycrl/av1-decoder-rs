pub trait BitRead {
    fn get_bits(&mut self, count: usize) -> u32;
    fn get_bit(&mut self) -> bool;
}

pub trait ExpGolomb {
    fn get_unsigned(&mut self) -> usize;
    fn get_signed(&mut self) -> isize;
}

#[derive(Default)]
pub struct Bits<'a> {
    buf: &'a [u8],
    index: usize,
    bit_pos: usize,
}

impl<'a> Bits<'a> {
    #[inline]
    pub fn new(buf: &'a [u8], shift_sub: usize) -> Bits<'a> {
        Self {
            buf,
            index: 0,
            bit_pos: shift_sub,
        }
    }

    #[inline]
    pub fn skip_next(&mut self) {
        let lz = self.count_leading_zeroes();
        self.skip_bits(lz);
    }

    #[inline]
    fn skip_bits(&mut self, num_bits: u32) {
        let offset = self.bit_pos + num_bits as usize;
        self.index = usize::min(self.buf.len(), self.index + offset / 8);
        self.bit_pos = offset % 8;
    }

    #[inline]
    fn count_leading_zeroes(&mut self) -> u32 {
        let mut leading_zeros = 0;
        for bit in self.by_ref() {
            if !bit {
                leading_zeros += 1;
                if leading_zeros > u64::BITS {
                    panic!()
                }
            } else {
                return leading_zeros;
            }
        }

        panic!()
    }
}

impl<'a> Iterator for Bits<'a> {
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

impl<'a> AsMut<Bits<'a>> for Bits<'a> {
    fn as_mut(&mut self) -> &mut Bits<'a> {
        self
    }
}

impl<'a> BitRead for Bits<'a> {
    #[inline]
    fn get_bits(&mut self, count: usize) -> u32 {
        let mut aac = 0;
        for i in 0..count {
            aac |= (self.get_bit() as u32) << (count - i - 1);
        }

        aac
    }

    #[inline]
    fn get_bit(&mut self) -> bool {
        self.next().unwrap_or(false)
    }
}

impl<'a> ExpGolomb for Bits<'a> {
    #[inline]
    fn get_unsigned(&mut self) -> usize {
        let mut lz = self.count_leading_zeroes();
        let x = (1u64.wrapping_shl(lz) - 1) as usize;
        let mut y = 0;

        if lz != 0 {
            for bit in self.by_ref() {
                y <<= 1;
                y |= bit as usize;
                lz -= 1;
                if lz == 0 {
                    break;
                }
            }

            if lz != 0 {
                panic!()
            }
        }

        x + y
    }

    #[inline]
    fn get_signed(&mut self) -> isize {
        let k = self.get_unsigned();
        let factor = if k % 2 == 0 { -1 } else { 1 };
        factor * (k / 2 + k % 2) as isize
    }
}
