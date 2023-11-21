mod sps;

use crate::{H264DecodeError, H264DecodeErrorKind};

use self::sps::Sps;

// 2bit
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
