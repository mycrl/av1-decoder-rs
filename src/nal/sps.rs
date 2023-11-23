use crate::utils::golomb::ExpGolombDecoder;

#[derive(Debug)]
pub enum SpsDecoderError {
    InvalidData,
    UnSupports,
}

impl std::error::Error for SpsDecoderError {}

impl std::fmt::Display for SpsDecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitDepth {
    Bit8,
    Bit9,
    Bit10,
    Bit11,
    Bit12,
    Bit13,
    Bit14,
}

impl TryFrom<u8> for BitDepth {
    type Error = SpsDecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Bit8,
            1 => Self::Bit9,
            2 => Self::Bit10,
            3 => Self::Bit11,
            4 => Self::Bit12,
            5 => Self::Bit13,
            6 => Self::Bit14,
            _ => return Err(SpsDecoderError::InvalidData),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaFormat {
    Yuv400,
    Yuv420,
    Yuv422,
    Yuv444,
}

impl TryFrom<u8> for ChromaFormat {
    type Error = SpsDecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Yuv400,
            1 => Self::Yuv420,
            2 => Self::Yuv422,
            3 => Self::Yuv444,
            _ => return Err(SpsDecoderError::InvalidData),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Baseline,
    Main,
    High,
    Extended,
}

impl TryFrom<u8> for Profile {
    type Error = SpsDecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            66 => Self::Baseline,
            77 => Self::Main,
            100 => Self::High,
            88 => Self::Extended,
            44 | 110 | 122 | 244 => return Err(SpsDecoderError::UnSupports),
            _ => return Err(SpsDecoderError::InvalidData),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PicOrderCntBasedOnFrameNumbers {
    pub delta_pic_order_always_zero_flag: bool,
    pub offset_for_non_ref_pic: i8,
    pub offset_for_top_to_bottom_field: i8,
    pub num_ref_frames_in_pic_order_cnt_cycle: u8,
    pub offset_for_ref_frame: Vec<i8>,
}

#[derive(Debug, Clone)]
pub enum PicOrderCnt {
    /// `log2_max_pic_order_cnt_lsb_minus4`
    None(u8),
    OnFrameNumbers(PicOrderCntBasedOnFrameNumbers),
    OnFieldNumbers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PicOrderCntType {
    None,
    OnFrameNumbers,
    OnFieldNumbers,
}

impl TryFrom<u8> for PicOrderCntType {
    type Error = SpsDecoderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::None,
            1 => Self::OnFrameNumbers,
            2 => Self::OnFieldNumbers,
            _ => return Err(SpsDecoderError::InvalidData),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameMbsOnly {
    FrameMode,
    /// `mb_adaptive_frame_field_flag`
    AdaptiveFrameFieldMode(bool),
}

#[derive(Debug, Clone, Copy)]
pub struct FrameCropping {
    pub left_offset: u8,
    pub right_offset: u8,
    pub top_offset: u8,
    pub bottom_offset: u8,
}

#[derive(Debug, Clone)]
pub struct Sps {
    pub profile_idc: Profile,
    pub constraint_setx_flags: [bool; 5],
    pub level_idc: u8,
    pub seq_parameter_set_id: u8,
    pub chroma_format_idc: ChromaFormat,
    pub separate_colour_plane_flag: bool,
    pub bit_depth_luma_minus8: BitDepth,
    pub bit_depth_chroma_minus8: BitDepth,
    pub qpprime_y_zero_transform_bypass_flag: bool,
    pub seq_scaling_matrix_present_flag: bool,
    pub log2_max_frame_num_minus4: u8,
    pub pic_order_cnt: PicOrderCnt,
    pub max_num_ref_frames: u8,
    pub gaps_in_frame_num_value_allowed_flag: bool,
    pub pic_width_in_mbs_minus1: u8,
    pub pic_height_in_map_units_minus1: u8,
    pub frame_mbs_only: FrameMbsOnly,
    pub direct_8x8_inference_flag: bool,
    pub frame_cropping: Option<FrameCropping>,
    pub vui_parameters_present_flag: bool,
}

impl TryFrom<&[u8]> for Sps {
    type Error = SpsDecoderError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = ExpGolombDecoder::new(value, 0);

        let profile_idc = Profile::try_from(reader.next_bits(8) as u8)?;
        let constraint_setx_flags = [
            reader.next_bit(),
            reader.next_bit(),
            reader.next_bit(),
            reader.next_bit(),
            reader.next_bit(),
        ];

        let level_idc = reader.next_bits(8) as u8;
        let seq_parameter_set_id = reader.next_unsigned();
        let chroma_format_idc = ChromaFormat::try_from(reader.next_unsigned())?;

        let mut separate_colour_plane_flag = false;
        if chroma_format_idc == ChromaFormat::Yuv444 {
            separate_colour_plane_flag = reader.next_bit();
        }

        let bit_depth_luma_minus8 = BitDepth::try_from(reader.next_unsigned())?;
        let bit_depth_chroma_minus8 = BitDepth::try_from(reader.next_unsigned())?;

        let qpprime_y_zero_transform_bypass_flag = reader.next_bit();
        let seq_scaling_matrix_present_flag = reader.next_bit();

        let mut scaling_list_4x4 = [[0u32; 16]; 6];
        let mut scaling_list_8x8 = [[0u32; 64]; 6];
        let mut use_default_scaling_matrix_flag_4x4 = [false; 6];
        let mut use_default_scaling_matrix_flag_8x8 = [false; 6];
        let mut seq_scaling_list_present_flag = [false; 12];
        if seq_scaling_matrix_present_flag {
            for i in 0..if chroma_format_idc != ChromaFormat::Yuv444 {
                8
            } else {
                12
            } {
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

        let log2_max_frame_num_minus4 = reader.next_unsigned();
        let pic_order_cnt_type = PicOrderCntType::try_from(reader.next_unsigned())?;
        let pic_order_cnt = match pic_order_cnt_type {
            PicOrderCntType::OnFieldNumbers => PicOrderCnt::OnFieldNumbers,
            PicOrderCntType::None => PicOrderCnt::None(reader.next_unsigned()),
            PicOrderCntType::OnFrameNumbers => {
                let mut ret = PicOrderCntBasedOnFrameNumbers {
                    delta_pic_order_always_zero_flag: reader.next_bit(),
                    offset_for_non_ref_pic: reader.next_signed(),
                    offset_for_top_to_bottom_field: reader.next_signed(),
                    num_ref_frames_in_pic_order_cnt_cycle: reader.next_unsigned(),
                    offset_for_ref_frame: vec![],
                };

                for _ in 0..ret.num_ref_frames_in_pic_order_cnt_cycle {
                    ret.offset_for_ref_frame.push(reader.next_signed());
                }

                PicOrderCnt::OnFrameNumbers(ret)
            }
        };

        let max_num_ref_frames = reader.next_unsigned();
        let gaps_in_frame_num_value_allowed_flag = reader.next_bit();
        let pic_width_in_mbs_minus1 = reader.next_unsigned();
        let pic_height_in_map_units_minus1 = reader.next_unsigned();

        let mut frame_mbs_only = FrameMbsOnly::FrameMode;
        let frame_mbs_only_flag = reader.next_bit();
        if !frame_mbs_only_flag {
            frame_mbs_only = FrameMbsOnly::AdaptiveFrameFieldMode(reader.next_bit());
        }

        let direct_8x8_inference_flag = reader.next_bit();

        let mut frame_cropping = None;
        let frame_cropping_flag = reader.next_bit();
        if frame_cropping_flag {
            frame_cropping = Some(FrameCropping {
                left_offset: reader.next_unsigned(),
                right_offset: reader.next_unsigned(),
                top_offset: reader.next_unsigned(),
                bottom_offset: reader.next_unsigned(),
            });
        }

        let vui_parameters_present_flag = reader.next_bit();
        if vui_parameters_present_flag {}

        Ok(Self {
            profile_idc,
            constraint_setx_flags,
            level_idc,
            seq_parameter_set_id,
            chroma_format_idc,
            separate_colour_plane_flag,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag,
            seq_scaling_matrix_present_flag,
            log2_max_frame_num_minus4,
            pic_order_cnt,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed_flag,
            pic_height_in_map_units_minus1,
            pic_width_in_mbs_minus1,
            frame_mbs_only,
            direct_8x8_inference_flag,
            frame_cropping,
            vui_parameters_present_flag,
        })
    }
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
