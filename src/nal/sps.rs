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
    pub chroma_format_idc: u8,
    pub separate_colour_plane_flag: Option<u8>,
}

impl TryFrom<&[u8]> for Sps {
    type Error = H264DecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = ExpGolombDecoder::new(value, 0);

        let profile_idc = Profile::try_from(reader.next_bits(8) as u8)?;

        let mut constraint_setx_flags = [0u8; 5];
        constraint_setx_flags[0] = reader.next_bit();
        constraint_setx_flags[1] = reader.next_bit();
        constraint_setx_flags[2] = reader.next_bit();
        constraint_setx_flags[3] = reader.next_bit();
        constraint_setx_flags[4] = reader.next_bit();

        let level_idc = reader.next_bits(8) as u8;
        let seq_parameter_set_id = reader.next_unsigned();
        let chroma_format_idc = reader.next_unsigned();

        let mut separate_colour_plane_flag = None;
        if chroma_format_idc == 3 {
            separate_colour_plane_flag = Some(reader.next_bits(1) as u8);
        }

        let bit_depth_luma_minus8 = reader.next_unsigned();
        let bit_depth_chroma_minus8 = reader.next_unsigned();
        let qpprime_y_zero_transform_bypass_flag = reader.next_bit();
        let seq_scaling_matrix_present = reader.next_bit();

        let mut seq_scaling_list_present_flag = [0u8; 12];
        if seq_scaling_matrix_present > 0 {
            for i in 0..if chroma_format_idc != 3 { 8 } else { 12 } {
                seq_scaling_list_present_flag[i] = reader.next_bit();
                if seq_scaling_list_present_flag[i] != 0 {
                    if i < 6 {
                        
                    } else {

                    }
                }
            }
        }

        Ok(Self {
            profile_idc,
            constraint_setx_flags,
            level_idc,
            seq_parameter_set_id,
            chroma_format_idc,
            separate_colour_plane_flag,
        })
    }
}
