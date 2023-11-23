mod sps;

use self::sps::{Sps, SpsDecodeError};

#[derive(Debug)]
pub enum NaluDecodeError {
    UnSupports,
    SpsDecodeError(SpsDecodeError),
}

impl std::error::Error for NaluDecodeError {}

impl std::fmt::Display for NaluDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// 2bit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nri {
    Disposable, // 0
    Low,        // 1
    High,       // 2
    Highest,    // 3
}

impl TryFrom<u8> for Nri {
    type Error = NaluDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match (value >> 5) & 3 {
            0 => Self::Disposable,
            1 => Self::Low,
            2 => Self::High,
            3 => Self::Highest,
            _ => return Err(NaluDecodeError::UnSupports),
        })
    }
}

// 5bit
//
// 0 - unused
// 13-23 - reserve
// 24-31 - unused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nut {
    Slice,                 // 1
    DataPartitoningSliceA, // 2
    DataPartitoningSliceB, // 3
    DataPartitoningSliceC, // 4
    IDRSlice,              // 5
    SEI,                   // 6
    SPS,                   // 7
    PPS,                   // 8
    Delimiter,             // 9
    EndOfSequence,         // 10
    EndOfCodeStream,       // 11
    Padding,               // 12
}

impl TryFrom<u8> for Nut {
    type Error = NaluDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value & 0x1F {
            1 => Self::Slice,
            2 => Self::DataPartitoningSliceA,
            3 => Self::DataPartitoningSliceB,
            4 => Self::DataPartitoningSliceC,
            5 => Self::IDRSlice,
            6 => Self::SEI,
            7 => Self::SPS,
            8 => Self::PPS,
            9 => Self::Delimiter,
            10 => Self::EndOfSequence,
            11 => Self::EndOfCodeStream,
            12 => Self::Padding,
            _ => return Err(NaluDecodeError::UnSupports),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Nalunit {
    SPS(Sps),
    // SEI,
    // PPS,
    // ISlice,
    // BSlice,
    // PSlice,
    // Delimiter,
}

#[derive(Debug, Clone)]
pub struct Nalu {
    pub ref_idc: Nri,
    pub unit: Nalunit,
}

impl TryFrom<&[u8]> for Nalu {
    type Error = NaluDecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let header = value[0];
        Ok(Self {
            ref_idc: Nri::try_from(header)?,
            unit: match Nut::try_from(header)? {
                Nut::SPS => Nalunit::SPS(
                    Sps::try_from(&value[1..]).map_err(|e| NaluDecodeError::SpsDecodeError(e))?,
                ),
                _ => todo!(),
            },
        })
    }
}

pub enum H264Package {
    Annexb(Nalu),
    RTP(Nalu),
}

#[cfg(test)]
mod tests {
    use super::Nalu;

    const SPS_NALU_BYTES: [u8; 34] = [
        0x00, 0x00, 0x00, 0x01, 0x67, 0x64, 0x00, 0x1F, 0xAC, 0xD9, 0x40, 0x50, 
        0x05, 0xBA, 0x6A, 0x02, 0x02, 0x03, 0x6E, 0x00, 0x00, 0x01, 0x00, 0x02, 
        0x00, 0x00, 0x01, 0x00, 0x60, 0x1E, 0x30, 0x63, 0x2C, 0x00
    ];

    #[test]
    fn parse_sps_nalu() {
        let nalu = Nalu::try_from(&SPS_NALU_BYTES[4..]).unwrap();
        println!("{:?}", nalu);
    }
}
