use bitreader::BitReader;

pub struct ExpGolombDecoder<'a> {
    reader: BitReader<'a>,
}

impl<'a> ExpGolombDecoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            reader: BitReader::new(buf),
        }
    }

    pub fn get_ue(&mut self) -> u8 {
        let mut i = 0;

        loop {
            let bit = self.get(1);
            if bit == 0 && i < 32 {
                i += 1;
            } else {
                break;
            }
        }

        let mut num = self.get(i);
        num += (1 << i) - 1;
        num
    }

    pub fn get_se(&mut self) -> i8 {
        let mut num = self.get_ue() as i8;
        if num & 0x01 > 0 {
            num = (num + 1) / 2;
        } else {
            num = num / 2;
            num = num - (num * 2);
        }

        num
    }

    pub fn get(&mut self, count: u8) -> u8 {
        self.reader
            .read_u8(count)
            .expect("golomb get bits is failed!")
    }
}
