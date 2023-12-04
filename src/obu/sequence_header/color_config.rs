use super::SequenceProfile;
use crate::{util::EasyAtomic, Av1DecodeError, Av1DecodeUnknownError, Av1DecoderContext, Buffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    Bt709,
    Unspecified,
    Bt470M,
    Bt470BG,
    Bt601,
    Smpte240,
    GenericFilm,
    Bt2020,
    Xyz,
    Smpte431,
    Smpte432,
    Ebu3213,
}

impl TryFrom<u8> for ColorPrimaries {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Bt709,
            2 => Self::Unspecified,
            4 => Self::Bt470M,
            5 => Self::Bt470BG,
            6 => Self::Bt601,
            7 => Self::Smpte240,
            8 => Self::GenericFilm,
            9 => Self::Bt2020,
            10 => Self::Xyz,
            11 => Self::Smpte431,
            12 => Self::Smpte432,
            22 => Self::Ebu3213,
            _ => {
                return Err(Av1DecodeError::Unknown(
                    Av1DecodeUnknownError::ColorPrimaries,
                ))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferCharacteristics {
    Bt709,
    Unspecified,
    Bt470M,
    Bt470BG,
    Bt601,
    Smpte240,
    Linear,
    Log100,
    Log100Sqrt10,
    Iec61966,
    Bt1361,
    Srgb,
    Bt202010Bit,
    Bt202012Bit,
    Smpte2084,
    Smpte428,
    Hlg,
}

impl TryFrom<u8> for TransferCharacteristics {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Bt709,
            2 => Self::Unspecified,
            4 => Self::Bt470M,
            5 => Self::Bt470BG,
            6 => Self::Bt601,
            7 => Self::Smpte240,
            8 => Self::Linear,
            9 => Self::Log100,
            10 => Self::Log100Sqrt10,
            11 => Self::Iec61966,
            12 => Self::Bt1361,
            13 => Self::Srgb,
            14 => Self::Bt202010Bit,
            15 => Self::Bt202012Bit,
            16 => Self::Smpte2084,
            17 => Self::Smpte428,
            18 => Self::Hlg,
            _ => {
                return Err(Av1DecodeError::Unknown(
                    Av1DecodeUnknownError::TransferCharacteristics,
                ))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixCoefficients {
    Identity,
    Bt709,
    Unspecified,
    Fcc,
    Bt470BG,
    Bt601,
    Smpte240,
    SmpteYcgco,
    Bt2020Ncl,
    Bt2020Cl,
    Smpte2085,
    ChromatNcl,
    ChromatCl,
    Ictcp,
}

impl TryFrom<u8> for MatrixCoefficients {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Identity,
            1 => Self::Bt709,
            2 => Self::Unspecified,
            4 => Self::Fcc,
            5 => Self::Bt470BG,
            6 => Self::Bt601,
            7 => Self::Smpte240,
            8 => Self::SmpteYcgco,
            9 => Self::Bt2020Ncl,
            10 => Self::Bt2020Cl,
            11 => Self::Smpte2085,
            12 => Self::ChromatNcl,
            13 => Self::ChromatCl,
            14 => Self::Ictcp,
            _ => {
                return Err(Av1DecodeError::Unknown(
                    Av1DecodeUnknownError::MatrixCoefficients,
                ))
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSamplePosition {
    Unknown,
    Vertical,
    Colocated,
}

impl TryFrom<u8> for ChromaSamplePosition {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Unknown,
            1 => Self::Vertical,
            2 => Self::Colocated,
            _ => {
                return Err(Av1DecodeError::Unknown(
                    Av1DecodeUnknownError::ChromaSamplePosition,
                ))
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub high_bitdepth: bool,
    pub twelve_bit: bool,
    pub mono_chrome: bool,
    pub color_description_present: bool,
    pub color_primaries: ColorPrimaries,
    pub transfer_characteristics: TransferCharacteristics,
    pub matrix_coefficients: MatrixCoefficients,
    pub color_range: bool,
    pub subsampling_x: bool,
    pub subsampling_y: bool,
    pub chroma_sample_position: Option<ChromaSamplePosition>,
    pub separate_uv_delta_q: bool,
}

impl ColorConfig {
    pub fn decode(
        ctx: &Av1DecoderContext,
        buf: &mut Buffer,
        profile: SequenceProfile,
    ) -> Result<Self, Av1DecodeError> {
        // high_bitdepth	f(1)
        let high_bitdepth = buf.get_bit();

        let mut twelve_bit = false;
        ctx.bit_depth.set(
            if profile == SequenceProfile::Professional && high_bitdepth {
                // twelve_bit	f(1)
                twelve_bit = buf.get_bit();
                if twelve_bit {
                    12
                } else {
                    10
                }
            } else {
                if high_bitdepth {
                    10
                } else {
                    8
                }
            },
        );

        let mono_chrome = if profile == SequenceProfile::Main {
            false
        } else {
            // mono_chrome	f(1)
            buf.get_bit()
        };

        ctx.num_planes.set(if mono_chrome { 1 } else { 3 });

        // color_description_present_flag	f(1)
        let color_description_present = buf.get_bit();
        let (color_primaries, transfer_characteristics, matrix_coefficients) =
            if color_description_present {
                (
                    // color_primaries	f(8)
                    ColorPrimaries::try_from(buf.get_bits(8) as u8)?,
                    // transfer_characteristics	f(8)
                    TransferCharacteristics::try_from(buf.get_bits(8) as u8)?,
                    // matrix_coefficients	f(8)
                    MatrixCoefficients::try_from(buf.get_bits(8) as u8)?,
                )
            } else {
                (
                    ColorPrimaries::Unspecified,
                    TransferCharacteristics::Unspecified,
                    MatrixCoefficients::Unspecified,
                )
            };

        let mut separate_uv_delta_q = false;
        let mut color_range = false;
        let mut subsampling_x = false;
        let mut subsampling_y = false;
        let mut chroma_sample_position = None;

        if mono_chrome {
            // color_range f(1)
            color_range = buf.get_bit();
            subsampling_x = true;
            subsampling_y = true;
            chroma_sample_position = Some(ChromaSamplePosition::Unknown);

            // TODO:
            // return
        } else if color_primaries == ColorPrimaries::Bt709
            && transfer_characteristics == TransferCharacteristics::Srgb
            && matrix_coefficients == MatrixCoefficients::Identity
        {
            color_range = true;
            subsampling_x = false;
            subsampling_y = false;
        } else {
            // color_range f(1)
            color_range = buf.get_bit();
            if profile == SequenceProfile::Main {
                subsampling_x = true;
                subsampling_y = true;
            } else if profile == SequenceProfile::High {
                subsampling_x = false;
                subsampling_y = false;
            } else {
                if ctx.bit_depth.get() == 12 {
                    // subsampling_x	f(1)
                    subsampling_x = buf.get_bit();
                    subsampling_y = if subsampling_x {
                        // subsampling_y	f(1)
                        buf.get_bit()
                    } else {
                        false
                    };
                } else {
                    subsampling_x = true;
                    subsampling_y = false;
                }
            }

            if subsampling_x && subsampling_y {
                // chroma_sample_position	f(2)
                chroma_sample_position =
                    Some(ChromaSamplePosition::try_from(buf.get_bits(2) as u8)?);
            }
        };

        // separate_uv_delta_q	f(1)
        separate_uv_delta_q = buf.get_bit();

        Ok(Self {
            high_bitdepth,
            twelve_bit,
            mono_chrome,
            color_description_present,
            color_primaries,
            transfer_characteristics,
            matrix_coefficients,
            color_range,
            subsampling_x,
            subsampling_y,
            chroma_sample_position,
            separate_uv_delta_q,
        })
    }
}
