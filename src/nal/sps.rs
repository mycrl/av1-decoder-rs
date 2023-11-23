use crate::utils::golomb::ExpGolombDecoder;

#[derive(Debug)]
pub enum SpsDecodeErrorKind {
    InvalidData,
    UnSupports,
}

#[derive(Debug)]
pub struct SpsDecodeError {
    pub kind: SpsDecodeErrorKind,
    pub help: &'static str,
}

impl std::error::Error for SpsDecodeError {}

impl std::fmt::Display for SpsDecodeError {
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
    type Error = SpsDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Bit8,
            1 => Self::Bit9,
            2 => Self::Bit10,
            3 => Self::Bit11,
            4 => Self::Bit12,
            5 => Self::Bit13,
            6 => Self::Bit14,
            _ => return Err(SpsDecodeError {
                kind: SpsDecodeErrorKind::InvalidData,
                help: "BitDepth"
            }),
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
    type Error = SpsDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Yuv400,
            1 => Self::Yuv420,
            2 => Self::Yuv422,
            3 => Self::Yuv444,
            _ => return Err(SpsDecodeError {
                kind: SpsDecodeErrorKind::InvalidData,
                help: "ChromaFormat"
            }),
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
    type Error = SpsDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            66 => Self::Baseline,
            77 => Self::Main,
            100 => Self::High,
            88 => Self::Extended,
            44 | 110 | 122 | 244 => return Err(SpsDecodeError {
                kind: SpsDecodeErrorKind::UnSupports,
                help: "Profile"
            }),
            _ => return Err(SpsDecodeError {
                kind: SpsDecodeErrorKind::InvalidData,
                help: "Profile"
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct PicOrderCntBasedOnFrameNumbers {
    pub delta_pic_order_always_zero_flag: bool,
    pub offset_for_non_ref_pic: isize,
    pub offset_for_top_to_bottom_field: isize,
    pub num_ref_frames_in_pic_order_cnt_cycle: usize,
    pub offset_for_ref_frame: Vec<isize>,
}

#[derive(Debug, Clone)]
pub enum PicOrderCnt {
    /// `log2_max_pic_order_cnt_lsb_minus4`
    None(usize),
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
    type Error = SpsDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::None,
            1 => Self::OnFrameNumbers,
            2 => Self::OnFieldNumbers,
            _ => return Err(SpsDecodeError {
                kind: SpsDecodeErrorKind::InvalidData,
                help: "PicOrderCntType"
            }),
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
    pub left_offset: usize,
    pub right_offset: usize,
    pub top_offset: usize,
    pub bottom_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectRatioInfoPresent {
    R1x1,
    R12x11,
    R10x11,
    R16x11,
    R40x33,
    R24x11,
    R20x11,
    R32x11,
    R80x33,
    R18x11,
    R15x11,
    R64x33,
    R160x99,
    R4x3,
    R3x2,
    R2x1,
    Extended {
        width: usize,
        height: usize,
    },
}

impl TryFrom<&mut ExpGolombDecoder<'_>> for AspectRatioInfoPresent {
    type Error = SpsDecodeError;

    fn try_from(value: &mut ExpGolombDecoder) -> Result<Self, Self::Error> {
        Ok(match value.get_bits(8) as u8 {
            1 => Self::R1x1,
            2 => Self::R12x11,
            3 => Self::R10x11,
            4 => Self::R16x11,
            5 => Self::R40x33,
            6 => Self::R24x11,
            7 => Self::R20x11,
            8 => Self::R32x11,
            9 => Self::R80x33,
            10 => Self::R18x11,
            11 => Self::R15x11,
            12 => Self::R64x33,
            13 => Self::R160x99,
            14 => Self::R4x3,
            15 => Self::R3x2,
            16 => Self::R2x1,
            255 => Self::Extended { 
                width: value.get_bits(16) as usize, 
                height: value.get_bits(16) as usize,
            },
            _ => return Err(SpsDecodeError {
                kind: SpsDecodeErrorKind::InvalidData,
                help: "Vui.AspectRatioInfoPresent"
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct Vui {
    pub aspect_ratio_info_present: AspectRatioInfoPresent,
}

#[derive(Debug, Clone)]
pub struct Sps {
    pub profile_idc: Profile,
    pub constraint_setx_flags: [bool; 6],
    pub level_idc: u32,
    pub seq_parameter_set_id: usize,
    pub chroma_format_idc: ChromaFormat,
    pub separate_colour_plane_flag: bool,
    pub bit_depth_luma_minus8: BitDepth,
    pub bit_depth_chroma_minus8: BitDepth,
    pub qpprime_y_zero_transform_bypass_flag: bool,
    pub seq_scaling_matrix_present_flag: bool,
    pub log2_max_frame_num_minus4: usize,
    pub pic_order_cnt: PicOrderCnt,
    pub max_num_ref_frames: usize,
    pub gaps_in_frame_num_value_allowed_flag: bool,
    pub pic_width_in_mbs_minus1: usize,
    pub pic_height_in_map_units_minus1: usize,
    pub frame_mbs_only: FrameMbsOnly,
    pub direct_8x8_inference_flag: bool,
    pub frame_cropping: Option<FrameCropping>,
    pub vui_parameters_present_flag: bool,
}

impl TryFrom<&[u8]> for Sps {
    type Error = SpsDecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = ExpGolombDecoder::new(value, 0);

        let profile_idc = Profile::try_from(reader.get_bits(8) as u8)?;
        let constraint_setx_flags = [
            reader.get_bit(),
            reader.get_bit(),
            reader.get_bit(),
            reader.get_bit(),
            reader.get_bit(),
            reader.get_bit(),
        ];

        reader.get_bits(2);

        let level_idc = reader.get_bits(8);
        let seq_parameter_set_id = reader.get_unsigned();
        let chroma_format_idc = ChromaFormat::try_from(reader.get_unsigned() as u8)?;

        let mut separate_colour_plane_flag = false;
        if chroma_format_idc == ChromaFormat::Yuv444 {
            separate_colour_plane_flag = reader.get_bit();
        }

        let bit_depth_luma_minus8 = BitDepth::try_from(reader.get_unsigned() as u8)?;
        let bit_depth_chroma_minus8 = BitDepth::try_from(reader.get_unsigned() as u8)?;

        let qpprime_y_zero_transform_bypass_flag = reader.get_bit();
        let seq_scaling_matrix_present_flag = reader.get_bit();

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
                seq_scaling_list_present_flag[i] = reader.get_bit();
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

        let log2_max_frame_num_minus4 = reader.get_unsigned();
        let pic_order_cnt_type = PicOrderCntType::try_from(reader.get_unsigned() as u8)?;
        let pic_order_cnt = match pic_order_cnt_type {
            PicOrderCntType::OnFieldNumbers => PicOrderCnt::OnFieldNumbers,
            PicOrderCntType::None => PicOrderCnt::None(reader.get_unsigned()),
            PicOrderCntType::OnFrameNumbers => {
                let mut ret = PicOrderCntBasedOnFrameNumbers {
                    delta_pic_order_always_zero_flag: reader.get_bit(),
                    offset_for_non_ref_pic: reader.get_signed(),
                    offset_for_top_to_bottom_field: reader.get_signed(),
                    num_ref_frames_in_pic_order_cnt_cycle: reader.get_unsigned(),
                    offset_for_ref_frame: vec![],
                };

                for _ in 0..ret.num_ref_frames_in_pic_order_cnt_cycle {
                    ret.offset_for_ref_frame.push(reader.get_signed());
                }

                PicOrderCnt::OnFrameNumbers(ret)
            }
        };

        let max_num_ref_frames = reader.get_unsigned();
        let gaps_in_frame_num_value_allowed_flag = reader.get_bit();
        let pic_width_in_mbs_minus1 = reader.get_unsigned();
        let pic_height_in_map_units_minus1 = reader.get_unsigned();

        let mut frame_mbs_only = FrameMbsOnly::FrameMode;
        let frame_mbs_only_flag = reader.get_bit();
        if !frame_mbs_only_flag {
            frame_mbs_only = FrameMbsOnly::AdaptiveFrameFieldMode(reader.get_bit());
        }

        let direct_8x8_inference_flag = reader.get_bit();

        let mut frame_cropping = None;
        let frame_cropping_flag = reader.get_bit();
        if frame_cropping_flag {
            frame_cropping = Some(FrameCropping {
                left_offset: reader.get_unsigned(),
                right_offset: reader.get_unsigned(),
                top_offset: reader.get_unsigned(),
                bottom_offset: reader.get_unsigned(),
            });
        }

        let vui_parameters_present_flag = reader.get_bit();
        if vui_parameters_present_flag {

        }

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
        let mut get_scale = 0;

        for i in 0..scaling_list.len() {
            if get_scale != 0 {
                let delta_scale = reader.get_signed();
                get_scale = (last_scale + delta_scale as u32 + 256) % 256;
                *use_default_scaling_matrix_flag = i == 0 && get_scale == 0;
            }

            scaling_list[i] = if get_scale == 0 {
                last_scale
            } else {
                get_scale
            };

            last_scale = scaling_list[i];
        }
    }

    fn parse_vui(reader: &mut ExpGolombDecoder) {

    }
}
