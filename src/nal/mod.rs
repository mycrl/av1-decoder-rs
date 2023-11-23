mod sps;

use crate::{H264DecodeError, H264DecodeErrorKind};

use self::sps::Sps;

// 2bit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nri {
    Disposable, // 0
    Low,        // 1
    High,       // 2
    Highest,    // 3
}

impl TryFrom<u8> for Nri {
    type Error = H264DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match (value >> 5) & 3 {
            0 => Self::Disposable,
            1 => Self::Low,
            2 => Self::High,
            3 => Self::Highest,
            _ => {
                return Err(H264DecodeError {
                    kind: H264DecodeErrorKind::UnSupports,
                    help: Some("nri only supports the range 0-3!"),
                })
            }
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
    type Error = H264DecodeError;

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
            _ => {
                return Err(H264DecodeError {
                    kind: H264DecodeErrorKind::UnSupports,
                    help: Some("unused or reserved word!"),
                })
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum NaluPayload {
    SPS(Sps),
    SEI,
    PPS,
    ISlice,
    BSlice,
    PSlice,
    Delimiter,
}

// impl<'a> TryFrom<&'a [u8]> for NaluPayload {
//     type Error = H264DecodeError;

//     fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
//         Ok(Self::SPS)
//     }
// }

pub struct Nalu {
    pub ref_idc: Nri,
    pub unit_type: Nut,
    pub payload: NaluPayload,
}

// impl TryFrom<&[u8]> for Nalu {
//     type Error = H264DecodeError;

//     fn try_from(mut value: &[u8]) -> Result<Self, Self::Error> {
//         let header = value.get_u8();
//         Ok(Self {
//             ref_idc: Nri::try_from(header)?,
//             unit_type: Nut::try_from(header)?,
//         })
//     }
// }

pub enum H264Package {
    Annexb(Nalu),
    RTP(Nalu),
}

mod tests {
    const SPS_NALU_BYTES: [u8; 64] = [
        0x00, 0x00, 0x00, 0x01, 0x67, 0x64, 0x00, 0x28, 0xAC, 0xD9, 0x40, 0x78, 
        0x02, 0x27, 0xE5, 0x9A, 0x80, 0x80, 0x80, 0xA0, 0x00, 0x00, 0x7D, 0x20, 
        0x00, 0x1D, 0x4C, 0x01, 0xE3, 0x06, 0x32, 0xC0, 0x00, 0x00, 0x00, 0x01, 
        0x68, 0xEB, 0xE3, 0xCB, 0x22, 0xC0, 0x00, 0x00, 0x01, 0x06, 0x05, 0xFF, 
        0xFF, 0xAB, 0xDC, 0x45, 0xE9, 0xBD, 0xE6, 0xD9, 0x48, 0xB7, 0x96, 0x2C,
        0xD8, 0x20, 0xD9, 0x23
    ];

    #[test]
    fn parse_sps_nalu() {

    }
}
