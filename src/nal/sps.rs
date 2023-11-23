use crate::{utils::golomb::ExpGolombDecoder, H264DecodeError, H264DecodeErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct Sps {
    pub profile_idc: Profile,
    pub constraint_setx_flags: [bool; 5],
    pub level_idc: u8,
    pub seq_parameter_set_id: u8,
    pub chroma_format_idc: u8,
    pub separate_colour_plane_flag: Option<u8>,
}

impl Sps {
    fn read_scaling_list(
        reader: &mut ExpGolombDecoder,
        scaling_list: &mut [u32],
        use_default_scaling_matrix_flag: &mut bool,
    ) {
        let mut last_scale = 8;
        let mut next_scale = 0;

        for i in 0..scaling_list.len() {
            if next_scale != 0 {
                let delta_scale = reader.next_signed();
                next_scale = (last_scale + delta_scale as u32 + 256) % 256;
                *use_default_scaling_matrix_flag = i == 0 && next_scale == 0;
            }

            scaling_list[i] = if next_scale == 0 {
                last_scale
            } else {
                next_scale
            };
            last_scale = scaling_list[i];
        }
    }
}

impl TryFrom<&[u8]> for Sps {
    type Error = H264DecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = ExpGolombDecoder::new(value, 0);

        let profile_idc = Profile::try_from(reader.next_bits(8) as u8)?;

        let mut constraint_setx_flags = [false; 5];
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

        let mut scaling_list_4x4 = [[0u32; 16]; 6];
        let mut scaling_list_8x8 = [[0u32; 64]; 6];
        let mut use_default_scaling_matrix_flag_4x4 = [false; 6];
        let mut use_default_scaling_matrix_flag_8x8 = [false; 6];
        let mut seq_scaling_list_present_flag = [false; 12];
        if seq_scaling_matrix_present {
            for i in 0..if chroma_format_idc != 3 { 8 } else { 12 } {
                seq_scaling_list_present_flag[i] = reader.next_bit();
                if seq_scaling_list_present_flag[i] {
                    if i < 6 {
                        Self::read_scaling_list(
                            &mut reader,
                            &mut scaling_list_4x4[i],
                            &mut use_default_scaling_matrix_flag_4x4[i],
                        );
                    } else {
                        Self::read_scaling_list(
                            &mut reader,
                            &mut scaling_list_8x8[i - 6],
                            &mut use_default_scaling_matrix_flag_8x8[i],
                        )
                    }
                }
            }
        }

        let mut chroma_array_type = 0;
        if separate_colour_plane_flag.is_none() {
            chroma_array_type = chroma_format_idc;
        }

        let mut offset_for_ref_frame = [0i8; 256];
        let mut num_ref_frames_in_pic_order_cnt_cycle = None;
        let mut offset_for_top_to_bottom_field = None;
        let mut offset_for_non_ref_pic = None;
        let mut delta_pic_order_always_zero_flag = None;
        let mut log2_max_frame_num_minus4 = reader.next_unsigned();
        let mut pic_order_cnt_type = reader.next_unsigned();
        if pic_order_cnt_type == 0 {
            log2_max_frame_num_minus4 = reader.next_unsigned();
        } else if pic_order_cnt_type == 1 {
            delta_pic_order_always_zero_flag = Some(reader.next_bit());
            offset_for_non_ref_pic = Some(reader.next_signed());
            offset_for_top_to_bottom_field = Some(reader.next_signed());
            num_ref_frames_in_pic_order_cnt_cycle = Some(reader.next_unsigned());

            for i in 0..num_ref_frames_in_pic_order_cnt_cycle.unwrap() {
                offset_for_ref_frame[i as usize] = reader.next_signed();
            }
        }

        let num_ref_frames = reader.next_unsigned();
        let gaps_in_frame_num_value_allowed_flag = reader.next_bit();
        let pic_width_in_mbs_minus1 = reader.next_unsigned();
        let pic_height_in_map_units_minus1 = reader.next_unsigned();
        let frame_mbs_only_flag = reader.next_bit();

        let mut mb_adaptive_frame_field_flag = None;
        if !frame_mbs_only_flag {
            mb_adaptive_frame_field_flag = Some(reader.next_bit());
        }

        let direct_8x8_inference_flag = reader.next_bit();
        let frame_cropping_flag = reader.next_bit();
        if frame_cropping_flag {
            let frame_crop_left_offset = reader.next_unsigned();
            let frame_crop_right_offset = reader.next_unsigned();
            let frame_crop_top_offset = reader.next_unsigned();
            let frame_crop_bottom_offset = reader.next_unsigned();
        }

        let vui_parameters_present_flag = reader.next_bit();
        if vui_parameters_present_flag {

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
