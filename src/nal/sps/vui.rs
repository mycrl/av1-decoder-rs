use crate::bitstream::{BitRead, Bits, ExpGolomb};

use super::{hrd::Hrd, SpsDecodeError, SpsDecodeErrorKind};

/// aspect_ratio_idc specifies the value of the sample aspect ratio of the luma
/// samples. Table E-1 shows the meaning of the code. When aspect_ratio_idc
/// indicates Extended_SAR, the sample aspect ratio is represented by
/// sar_width : sar_height. When the aspect_ratio_idc syntax element is not
/// present, aspect_ratio_idc value shall be inferred to be equal to 0.
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
        /// sar_width indicates the horizontal size of the sample aspect ratio
        /// (in arbitrary units).
        width: u16,
        /// sar_height indicates the vertical size of the sample aspect ratio
        /// (in the same arbitrary units as sar_width).
        height: u16,
    },
}

impl TryFrom<&mut Bits<'_>> for AspectRatioInfoPresent {
    type Error = SpsDecodeError;

    fn try_from(value: &mut Bits) -> Result<Self, Self::Error> {
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
                // sar_width 0 u(16)
                width: value.get_bits(16) as u16,
                // sar_height 0 u(16)
                height: value.get_bits(16) as u16,
            },
            _ => {
                return Err(SpsDecodeError {
                    kind: SpsDecodeErrorKind::InvalidData,
                    help: "Vui.AspectRatioInfoPresent",
                })
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OverscanInfoPresent {
    /// overscan_appropriate_flag equal to 1 indicates that the cropped decoded
    /// pictures output are suitable for display using
    /// overscan. overscan_appropriate_flag equal to 0 indicates that the
    /// cropped decoded pictures output contain visually
    /// important information in the entire region out to the edges of the
    /// cropping rectangle of the picture, such that the cropped
    /// decoded pictures output should not be displayed using overscan. Instead,
    /// they should be displayed using either an exact match between the
    /// display area and the cropping rectangle, or using underscan. As used in
    /// this paragraph, the term "overscan" refers to display processes in
    /// which some parts near the borders of the cropped decoded pictures are
    /// not visible in the display area. The term "underscan" describes
    /// display processes in which the entire cropped decoded pictures are
    /// visible in the display area, but they do not cover the entire display
    /// area. For display processes that neither use overscan nor underscan,
    /// the display area exactly matches the area of the cropped decoded
    /// pictures.
    pub overscan_appropriate: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ColourDescriptionPresent {
    /// colour_primaries indicates the chromaticity coordinates of the source
    /// primaries as specified in Table E-3 in terms of the CIE 1931
    /// definition of x and y as specified by ISO/CIE 10527.
    pub colour_primaries: u8,
    /// transfer_characteristics indicates the opto-electronic transfer
    /// characteristic of the source picture as specified in Table E-4 as a
    /// function of a linear optical intensity input Lc with a real-valued range
    /// of 0 to 1.
    pub transfer_characteristics: u8,
    /// matrix_coefficients describes the matrix coefficients used in deriving
    /// luma and chroma signals from the green, blue, and red primaries, as
    /// specified in Table E-5.
    pub matrix_coefficients: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct VideoSignalTypePresent {
    /// video_format indicates the representation of the pictures as specified
    /// in Table E-2, before being coded in accordance
    /// with this Recommendation | International Standard. When the video_format
    /// syntax element is not present, video_format value shall be inferred
    /// to be equal to 5.
    pub video_format: u8,
    /// video_full_range_flag indicates the black level and range of the luma
    /// and chroma signals as derived from E′Y, E′PB, and E′PR or E′R, E′G,
    /// and E′B real-valued component signals.
    ///
    /// When the video_full_range_flag syntax element is not present, the value
    /// of video_full_range_flag shall be inferred to be equal to 0.
    pub video_full_range: bool,
    pub colour_description_present: Option<ColourDescriptionPresent>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChromaLocInfoPresent {
    /// chroma_loc_info_present_flag equal to 1 specifies that
    /// chroma_sample_loc_type_top_field and
    /// chroma_sample_loc_type_bottom_field are present.
    /// chroma_loc_info_present_flag equal to 0 specifies that
    /// chroma_sample_loc_type_top_field and chroma_sample_loc_type_bottom_field
    /// are not present.
    pub chroma_sample_loc_type_top_field: usize,
    /// chroma_sample_loc_type_top_field and chroma_sample_loc_type_bottom_field
    /// specify the location of chroma samples as follows.
    ///
    /// – If chroma_format_idc is equal to 1 (4:2:0 chroma format),
    /// chroma_sample_loc_type_top_field and
    /// chroma_sample_loc_type_bottom_field specify the location of chroma
    /// samples for the top field and the bottom field, respectively, as
    /// shown in Figure E-1.
    ///
    /// – Otherwise (chroma_format_idc is not equal to 1), the values of the
    /// syntax elements chroma_sample_loc_type_top_field and
    /// chroma_sample_loc_type_bottom_field shall be ignored. When
    /// chroma_format_idc is equal to 2 (4:2:2 chroma format) or 3 (4:4:4 chroma
    /// format), the location of chroma samples is specified in subclause
    /// 6.2. When chroma_format_idc is equal to 0, there is no chroma sample
    /// array.
    ///
    /// The value of chroma_sample_loc_type_top_field and
    /// chroma_sample_loc_type_bottom_field shall be in the range of 0 to 5,
    /// inclusive. When the chroma_sample_loc_type_top_field and
    /// chroma_sample_loc_type_bottom_field are not present, the values of
    /// chroma_sample_loc_type_top_field and chroma_sample_loc_type_bottom_field
    /// shall be inferred to be equal to 0.
    pub chroma_sample_loc_type_bottom_field: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct TimingInfoPresent {
    /// num_units_in_tick is the number of time units of a clock operating at
    /// the frequency time_scale Hz that corresponds to one increment
    /// (called a clock tick) of a clock tick counter. num_units_in_tick shall
    /// be greater than 0. A clock tick is the minimum interval of time that
    /// can be represented in the coded data. For example, when the frame rate
    /// of a video signal is 30 000 ÷ 1001 Hz, time_scale may be equal to 60
    /// 000 and num_units_in_tick may be equal to 1001. See Equation C-1.
    pub num_units_in_tick: u32,
    /// time_scale is the number of time units that pass in one second. For
    /// example, a time coordinate system that measures time using a 27 MHz
    /// clock has a time_scale of 27 000 000. time_scale shall be greater than
    /// 0.
    pub time_scale: u32,
    /// fixed_frame_rate_flag equal to 1 indicates that the temporal distance
    /// between the HRD output times of any two consecutive pictures in
    /// output order is constrained as follows. fixed_frame_rate_flag equal to 0
    /// indicates that no such constraints apply to the temporal distance
    /// between the HRD output times of any two consecutive pictures in output
    /// order.
    pub fixed_frame_rate: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BitstreamRestriction {
    /// motion_vectors_over_pic_boundaries_flag equal to 0 indicates that no
    /// sample outside the picture boundaries and no sample at a fractional
    /// sample position whose value is derived using one or more samples outside
    /// the picture boundaries is used to inter predict any sample.
    /// motion_vectors_over_pic_boundaries_flag equal to 1 indicates that one or
    /// more samples outside picture boundaries may be used in inter
    /// prediction. When the motion_vectors_over_pic_boundaries_flag syntax
    /// element is not present, motion_vectors_over_pic_boundaries_flag
    /// value shall be inferred to be equal to 1.
    pub motion_vectors_over_pic_boundaries: bool,
    /// max_bytes_per_pic_denom indicates a number of bytes not exceeded by the
    /// sum of the sizes of the VCL NAL units associated with any coded
    /// picture in the coded video sequence. The number of bytes that
    /// represent a picture in the NAL unit stream is specified for this purpose
    /// as the total number of bytes of VCL NAL unit data (i.e., the total
    /// of the NumBytesInNALunit variables for the VCL NAL units) for the
    /// picture. The value of max_bytes_per_pic_denom shall be in the range of 0
    /// to 16, inclusive
    pub max_bytes_per_pic_denom: usize,
    /// max_bits_per_mb_denom indicates an upper bound for the number of coded
    /// bits of macroblock_layer( ) data for any macroblock in any picture
    /// of the coded video sequence. The value of max_bits_per_mb_denom shall be
    /// in the range of 0 to 16, inclusive.
    pub max_bits_per_mb_denom: usize,
    /// log2_max_mv_length_horizontal and log2_max_mv_length_vertical indicate
    /// the maximum absolute value of a decoded horizontal and vertical
    /// motion vector component, respectively, in ¼ luma sample units, for all
    /// pictures in the coded video sequence. A value of n asserts that no
    /// value of a motion vector component shall exceed the range from −2n
    /// to 2n − 1, inclusive, in units of ¼ luma sample displacement. The value
    /// of log2_max_mv_length_horizontal shall be in the range of 0 to 16,
    /// inclusive. The value of log2_max_mv_length_vertical shall be in the
    /// range of 0 to 16, inclusive. When log2_max_mv_length_horizontal is
    /// not present, the values of log2_max_mv_length_horizontal and
    /// log2_max_mv_length_vertical shall be inferred to be equal to 16
    pub log2_max_mv_length_horizontal: usize,
    pub log2_max_mv_length_vertical: usize,
    /// num_reorder_frames indicates the maximum number of frames, complementary
    /// field pairs, or non-paired fields that precede any frame,
    /// complementary field pair, or non-paired field in the coded video
    /// sequence in decoding order and follow it in output order. The value
    /// of num_reorder_frames shall be in the range of 0 to
    /// max_dec_frame_buffering, inclusive. When the num_reorder_frames
    /// syntax element is not present, the value of num_reorder_frames value
    /// shall be inferred as follows.
    ///
    /// – If profile_idc is equal to 44, 86, 100, 110, 122, or 244 and
    /// constraint_set3_flag is equal to 1, the value of num_reorder_frames
    /// shall be inferred to be equal to 0.
    ///
    /// – Otherwise (profile_idc is not equal to 44, 86, 100, 110, 122, or 244
    /// or constraint_set3_flag is equal to 0), the value
    /// of num_reorder_frames shall be inferred to be equal to MaxDpbFrames.
    pub num_reorder_frames: usize,
    /// max_dec_frame_buffering specifies the required size of the HRD decoded
    /// picture buffer (DPB) in units of frame buffers. The coded video
    /// sequence shall not require a decoded picture buffer with size of more
    /// than Max( 1, max_dec_frame_buffering ) frame buffers to enable the
    /// output of decoded pictures at the output times specified
    /// by dpb_output_delay of the picture timing SEI messages. The value of
    /// max_dec_frame_buffering shall be greater than or equal to
    /// max_num_ref_frames. An upper limit for the value of
    /// max_dec_frame_buffering is specified by the level
    /// limits in subclauses A.3.1, A.3.2, G.10.2.1, and H.10.2.
    ///
    /// When the max_dec_frame_buffering syntax element is not present, the
    /// value of max_dec_frame_buffering shall be inferred as follows.
    ///
    /// – If profile_idc is equal to 44, 86, 100, 110, 122, or 244 and
    /// constraint_set3_flag is equal to 1, the value of
    /// max_dec_frame_buffering shall be inferred to be equal to 0.
    ///
    /// – Otherwise (profile_idc is not equal to 44, 86, 100, 110, 122, or 244
    /// or constraint_set3_flag is equal to 0), the value
    /// of max_dec_frame_buffering shall be inferred to be equal to
    /// MaxDpbFrames.
    pub max_dec_frame_buffering: usize,
}

#[derive(Debug, Clone)]
pub struct Vui {
    pub aspect_ratio_info_present: Option<AspectRatioInfoPresent>,
    pub overscan_info_present: Option<OverscanInfoPresent>,
    pub video_signal_type_present: Option<VideoSignalTypePresent>,
    pub chroma_loc_info_present: Option<ChromaLocInfoPresent>,
    pub timing_info_present: Option<TimingInfoPresent>,
    pub nal_hrd_parameters_present: Option<Hrd>,
    pub vcl_hrd_parameters_present: Option<Hrd>,
    /// low_delay_hrd_flag specifies the HRD operational mode as specified in
    /// Annex C. When fixed_frame_rate_flag is equal to 1,
    /// low_delay_hrd_flag shall be equal to 0. When low_delay_hrd_flag is not
    /// present, its value shall be inferred to be equal to 1 −
    /// fixed_frame_rate_flag.
    pub low_delay_hrd: Option<bool>,
    /// pic_struct_present_flag equal to 1 specifies that picture timing SEI
    /// messages (subclause D.2.2) are present that include the pic_struct
    /// syntax element. pic_struct_present_flag equal to 0 specifies that the
    /// pic_struct syntax element is not present in picture timing SEI
    /// messages. When pic_struct_present_flag is not present, its value shall
    /// be inferred to be equal to 0
    pub pic_struct_present: bool,
    pub bitstream_restriction: Option<BitstreamRestriction>,
}

impl TryFrom<&mut Bits<'_>> for Vui {
    type Error = SpsDecodeError;

    fn try_from(value: &mut Bits) -> Result<Self, Self::Error> {
        // aspect_ratio_info_present_flag 0 u(1)
        let aspect_ratio_info_present = if value.get_bit() {
            // aspect_ratio_idc 0 u(8)
            Some(AspectRatioInfoPresent::try_from(value.as_mut())?)
        } else {
            None
        };

        // overscan_info_present_flag 0 u(1)
        let overscan_info_present = if value.get_bit() {
            Some(OverscanInfoPresent {
                // overscan_appropriate_flag 0 u(1)
                overscan_appropriate: value.get_bit(),
            })
        } else {
            None
        };

        // video_signal_type_present_flag 0 u(1)
        let video_signal_type_present = if value.get_bit() {
            Some(VideoSignalTypePresent {
                // video_format 0 u(3)
                video_format: value.get_bits(3) as u8,
                // video_full_range_flag 0 u(1)
                video_full_range: value.get_bit(),
                // colour_description_present_flag 0 u(1)
                colour_description_present: if value.get_bit() {
                    Some(ColourDescriptionPresent {
                        // colour_primaries 0 u(8)
                        colour_primaries: value.get_bits(8) as u8,
                        // transfer_characteristics 0 u(8)
                        transfer_characteristics: value.get_bits(8) as u8,
                        // matrix_coefficients 0 u(8)
                        matrix_coefficients: value.get_bits(8) as u8,
                    })
                } else {
                    None
                },
            })
        } else {
            None
        };

        // chroma_loc_info_present_flag 0 u(1)
        let chroma_loc_info_present = if value.get_bit() {
            Some(ChromaLocInfoPresent {
                // chroma_sample_loc_type_top_field 0 ue(v)
                chroma_sample_loc_type_top_field: value.get_unsigned(),
                // chroma_sample_loc_type_bottom_field 0 ue(v)
                chroma_sample_loc_type_bottom_field: value.get_unsigned(),
            })
        } else {
            None
        };

        // timing_info_present_flag 0 u(1)
        let timing_info_present = if value.get_bit() {
            Some(TimingInfoPresent {
                // num_units_in_tick 0 u(32)
                num_units_in_tick: value.get_bits(32),
                // time_scale 0 u(32)
                time_scale: value.get_bits(32),
                // fixed_frame_rate_flag 0 u(1)
                fixed_frame_rate: value.get_bit(),
            })
        } else {
            None
        };

        // nal_hrd_parameters_present_flag 0 u(1)
        let nal_hrd_parameters_present_flag = value.get_bit();
        let nal_hrd_parameters_present = if nal_hrd_parameters_present_flag {
            Some(Hrd::try_from(value.as_mut())?)
        } else {
            None
        };

        // vcl_hrd_parameters_present_flag 0 u(1)
        let vcl_hrd_parameters_present_flag = value.get_bit();
        let vcl_hrd_parameters_present = if vcl_hrd_parameters_present_flag {
            Some(Hrd::try_from(value.as_mut())?)
        } else {
            None
        };

        let low_delay_hrd = if nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag {
            // low_delay_hrd_flag 0 u(1)
            Some(value.get_bit())
        } else {
            None
        };

        // pic_struct_present_flag 0 u(1)
        let pic_struct_present = value.get_bit();

        // bitstream_restriction_flag 0 u(1)
        let bitstream_restriction = if value.get_bit() {
            Some(BitstreamRestriction {
                // motion_vectors_over_pic_boundaries_flag 0 u(1)
                motion_vectors_over_pic_boundaries: value.get_bit(),
                // max_bytes_per_pic_denom 0 ue(v)
                max_bytes_per_pic_denom: value.get_unsigned(),
                // max_bits_per_mb_denom 0 ue(v)
                max_bits_per_mb_denom: value.get_unsigned(),
                // log2_max_mv_length_horizontal 0 ue(v)
                log2_max_mv_length_horizontal: value.get_unsigned(),
                // log2_max_mv_length_vertical 0 ue(v)
                log2_max_mv_length_vertical: value.get_unsigned(),
                // num_reorder_frames 0 ue(v)
                num_reorder_frames: value.get_unsigned(),
                // max_dec_frame_buffering 0 ue(v)
                max_dec_frame_buffering: value.get_unsigned(),
            })
        } else {
            None
        };

        Ok(Self {
            aspect_ratio_info_present,
            overscan_info_present,
            video_signal_type_present,
            chroma_loc_info_present,
            timing_info_present,
            nal_hrd_parameters_present,
            vcl_hrd_parameters_present,
            low_delay_hrd,
            pic_struct_present,
            bitstream_restriction,
        })
    }
}
