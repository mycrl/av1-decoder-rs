#[derive(Debug)]
pub enum H264DecodeErrorKind {
    UnSupports,
}

impl std::fmt::Display for H264DecodeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub struct H264DecodeError {
    pub kind: H264DecodeErrorKind,
    pub help: Option<&'static str>,
}

impl H264DecodeError {
    fn default_from(kind: H264DecodeErrorKind) -> Self {
        Self { kind, help: None }
    }
}

impl std::error::Error for H264DecodeError {}

impl std::fmt::Display for H264DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.help {
            Some(help) => write!(f, "{} - {}", self.kind, help),
            None => write!(f, "{}", self.kind),
        }
    }
}

// 2bit
pub enum Nri {
    Unimportant, // 0
    Normal,      // 1
    Priority,    // 2
    Important,   // 3
}

impl TryFrom<u8> for Nri {
    type Error = H264DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match (value & 0b00011111u8) >> 5 {
            0 => Self::Unimportant,
            1 => Self::Normal,
            2 => Self::Priority,
            3 => Self::Important,
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
        Ok(match (value & 0b11100000u8) >> 5 {
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

pub enum Rbsp {
    SPS,
    SEI,
    PPS,
    ISlice,
    BSlice,
    PSlice,
    Delimiter,
}

impl<'a> TryFrom<&'a [u8]> for Rbsp {
    type Error = H264DecodeError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        

        Ok(Self::SPS)
    }
}

pub struct Nalu {
    ref_idc: Nri,
    unit_type: Nut,
    rbsp: Rbsp,
}

pub enum H264Package {
    Annexb(Nalu),
    RTP(Nalu),
}
