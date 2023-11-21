use bytes::Buf;

use crate::{utils::golomb::ExpGolombDecoder, H264DecodeError, H264DecodeErrorKind};

pub enum Profile {
    Baseline,
    Main,
    High,
    Extended,
}

impl TryFrom<u8> for Profile {
    type Error = H264DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            66 => Self::Baseline,
            77 => Self::Main,
            100 => Self::High,
            88 => Self::Extended,
            44 | 110 | 122 | 244 => {
                return Err(H264DecodeError {
                    kind: H264DecodeErrorKind::UnSupports,
                    help: Some("only supports baseline/main/high/extended profiles!"),
                })
            }
            _ => {
                return Err(H264DecodeError {
                    kind: H264DecodeErrorKind::UnknownData,
                    help: None,
                })
            }
        })
    }
}

pub struct Sps {
    pub profile_idc: Profile,
    pub constraint_setx_flags: [u8; 5],
    pub level_idc: u8,
    pub seq_parameter_set_id: u8,
}

impl TryFrom<&[u8]> for Sps {
    type Error = H264DecodeError;

    fn try_from(mut value: &[u8]) -> Result<Self, Self::Error> {
        let profile_idc = Profile::try_from(value.get_u8())?;

        let constraint_setx_byte = [value.get_u8()];
        let mut constraint_setx_flags = [0u8; 5];
        let mut reader = ExpGolombDecoder::new(&constraint_setx_byte);
        constraint_setx_flags[0] = reader.get(1);
        constraint_setx_flags[1] = reader.get(1);
        constraint_setx_flags[2] = reader.get(1);
        constraint_setx_flags[3] = reader.get(1);
        constraint_setx_flags[4] = reader.get(1);

        let level_idc = value.get_u8();
        let seq_parameter_set_id = value.get_u8();

        Ok(Self {
            profile_idc,
            constraint_setx_flags,
            level_idc,
            seq_parameter_set_id,
        })
    }
}
