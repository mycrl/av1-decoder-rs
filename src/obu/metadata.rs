use crate::{Av1DecodeError, Av1DecodeUnknownError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataType {
    HdrCll,
    HdrMdcv,
    Scalability,
    ItutT35,
    Timecode,
    UnregisteredUserPrivate,
}

impl TryFrom<u8> for MetadataType {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::HdrCll,
            2 => Self::HdrMdcv,
            3 => Self::Scalability,
            4 => Self::ItutT35,
            5 => Self::Timecode,
            6..=31 => Self::UnregisteredUserPrivate,
            _ => {
                return Err(Av1DecodeError::Unknown(
                    Av1DecodeUnknownError::ChromaSamplePosition,
                ))
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct ItutT35 {
    pub country_code: u8,
    pub country_code_extension_byte: Option<u8>,
    pub payload_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Metadata {
    // HdrCll,
    // HdrMdcv,
    // Scalability,
    // ItutT35,
    // Timecode,
    // UnregisteredUserPrivate(u8),
}
