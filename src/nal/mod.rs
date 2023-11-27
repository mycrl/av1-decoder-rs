pub mod pps;
pub mod sps;

use crate::bitstream::{BitRead, Bits};

use thiserror::Error;

use self::{
    pps::{Pps, PpsDecodeError},
    sps::{Sps, SpsDecodeError},
};

#[derive(Error, Debug)]
pub enum NaluDecodeError {
    UnSupports,
    SpsDecodeError(#[from] SpsDecodeError),
    PpsDecodeError(#[from] PpsDecodeError),
}

impl std::fmt::Display for NaluDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// 2bit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nri {
    Disposable,
    Low,
    High,
    Highest,
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
    Slice,
    DataPartitoningSliceA,
    DataPartitoningSliceB,
    DataPartitoningSliceC,
    IDRSlice,
    SEI,
    SPS,
    PPS,
    Delimiter,
    EndOfSequence,
    EndOfCodeStream,
    Padding,
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
    Sps(Sps),
    Pps(Pps),
    // SEI,
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
        let mut bits = Bits::new(value, 0);

        let header = bits.get_bits(8) as u8;
        Ok(Self {
            ref_idc: Nri::try_from(header)?,
            unit: match Nut::try_from(header)? {
                Nut::SPS => Nalunit::Sps(Sps::try_from(&mut bits)?),
                Nut::PPS => Nalunit::Pps(Pps::try_from(&mut bits)?),
                _ => todo!(),
            },
        })
    }
}
