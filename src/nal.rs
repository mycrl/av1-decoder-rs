// 2bit
pub enum NRI {
    Unimportant, // 0
    Normal,      // 1
    Priority,    // 2
    Important,   // 3
}

impl From<u8> for NRI {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unimportant,
            1 => Self::Normal,
            2 => Self::Priority,
            3 => Self::Important,
            _ => unreachable!("nri only supports the range 0-3!"),
        }
    }
}

// 5bit
//
// 0 - unused
// 13-23 - reserve
// 24-31 - unused
pub enum NUT {
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

impl From<u8> for NUT {
    fn from(value: u8) -> Self {
        match value {
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
            _ => unreachable!("unused or reserved word!"),
        }
    }
}

pub enum RBSP {
    SPS,
    SEI,
    PPS,
    ISlice,
    BSlice,
    PSlice,
    Delimiter,
}

pub struct NALU {
    ref_idc: NRI,
    unit_type: NUT,
}

pub enum H264Package {
    Annexb(NALU),
    RTP(NALU),
}
