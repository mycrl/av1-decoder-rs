mod vui;

use crate::bitstream::{BitRead, Bits, ExpGolomb};

use self::vui::Vui;

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
            _ => {
                return Err(SpsDecodeError {
                    kind: SpsDecodeErrorKind::InvalidData,
                    help: "BitDepth",
                })
            }
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
            _ => {
                return Err(SpsDecodeError {
                    kind: SpsDecodeErrorKind::InvalidData,
                    help: "ChromaFormat",
                })
            }
        })
    }
}

/// profile_idc and level_idc indicate the profile and level to which the coded
/// video sequence conforms.
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
            44 | 110 | 122 | 244 => {
                return Err(SpsDecodeError {
                    kind: SpsDecodeErrorKind::UnSupports,
                    help: "Profile",
                })
            }
            _ => {
                return Err(SpsDecodeError {
                    kind: SpsDecodeErrorKind::InvalidData,
                    help: "Profile",
                })
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct PicOrderCntBasedOnFrameNumbers {
    pub delta_pic_order_always_zero: bool,
    pub offset_for_non_ref_pic: isize,
    pub offset_for_top_to_bottom_field: isize,
    pub num_ref_frames_in_pic_order_cnt_cycle: usize,
    pub offset_for_ref_frame: Vec<isize>,
}

/// pic_order_cnt_type specifies the method to decode picture order count (as
/// specified in subclause 8.2.1). The value of pic_order_cnt_type shall be in
/// the range of 0 to 2, inclusive.
///
/// pic_order_cnt_type shall not be equal to 2 in a coded video sequence that
/// contains any of the following:
///
/// – an access unit containing a non-reference frame followed immediately by an
/// access unit containing a nonreference picture,
///
/// – two access units each containing a field with the two fields together
/// forming a complementary non-reference field pair followed immediately by an
/// access unit containing a non-reference picture,
///
/// – an access unit containing a non-reference field followed immediately by an
/// access unit containing another nonreference picture that does not form a
/// complementary non- reference field pair with the first of the two access
/// units.
#[derive(Debug, Clone)]
pub enum PicOrderCnt {
    None {
        /// log2_max_pic_order_cnt_lsb_minus4 specifies the value of the
        /// variable MaxPicOrderCntLsb that is used in the
        /// decoding process for picture order count as specified in subclause
        /// 8.2.1 as follows:
        ///
        /// MaxPicOrderCntLsb = 2( log2_max_pic_order_cnt_lsb_minus4 + 4 )
        ///
        /// The value of log2_max_pic_order_cnt_lsb_minus4 shall be in the range
        /// of 0 to 12, inclusive.
        log2_max_pic_order_cnt_lsb_minus4: usize,
    },
    OnFrameNumbers {
        /// delta_pic_order_always_zero_flag equal to 1 specifies that
        /// delta_pic_order_cnt[ 0 ] and delta_pic_order_cnt[ 1 ] are
        /// not present in the slice headers of the sequence and shall be
        /// inferred to be equal to 0. delta_pic_order_always_zero_flag
        /// equal to 0 specifies that delta_pic_order_cnt[ 0 ] is present in the
        /// slice headers of the sequence and delta_pic_order_cnt[ 1 ]
        /// may be present in the slice headers of the sequence.
        delta_pic_order_always_zero: bool,
        /// offset_for_non_ref_pic is used to calculate the picture order count
        /// of a non-reference picture as specified in subclause 8.2.1.
        /// The value of offset_for_non_ref_pic shall be in the range of −231 +
        /// 1 to 231 − 1, inclusive.
        offset_for_non_ref_pic: isize,
        /// offset_for_top_to_bottom_field is used to calculate the picture
        /// order count of a bottom field as specified in subclause 8.2.
        /// 1. The value of offset_for_top_to_bottom_field shall be in the range
        /// of −231 + 1 to 231 − 1, inclusive
        offset_for_top_to_bottom_field: isize,
        /// num_ref_frames_in_pic_order_cnt_cycle is used in the decoding
        /// process for picture order count as specified in subclause 8.
        /// 2.1. The value of num_ref_frames_in_pic_order_cnt_cycle shall be in
        /// the range of 0 to 255, inclusive.
        num_ref_frames_in_pic_order_cnt_cycle: usize,
        /// offset_for_ref_frame[ i ] is an element of a list of
        /// num_ref_frames_in_pic_order_cnt_cycle values used in the
        /// decoding process for picture order count as specified in subclause
        /// 8.2.1. The value of offset_for_ref_frame[ i ] shall be
        /// in the range of −231 + 1 to 231 − 1, inclusive.
        /// When pic_order_cnt_type is equal to 1.
        offset_for_ref_frame: Vec<isize>,
    },
    OnFieldNumbers,
}

impl TryFrom<&mut Bits<'_>> for PicOrderCnt {
    type Error = SpsDecodeError;

    fn try_from(value: &mut Bits) -> Result<Self, Self::Error> {
        Ok(match value.get_unsigned() as u8 {
            0 => Self::None {
                // log2_max_pic_order_cnt_lsb_minus4 0 ue(v)
                log2_max_pic_order_cnt_lsb_minus4: value.get_unsigned(),
            },
            1 => {
                // delta_pic_order_always_zero_flag 0 u(1)
                let delta_pic_order_always_zero = value.get_bit();

                // offset_for_non_ref_pic 0 se(v)
                let offset_for_non_ref_pic = value.get_signed();

                // offset_for_top_to_bottom_field 0 se(v)
                let offset_for_top_to_bottom_field = value.get_signed();

                // num_ref_frames_in_pic_order_cnt_cycle 0 ue(v)
                let num_ref_frames_in_pic_order_cnt_cycle = value.get_unsigned();

                let mut offset_for_ref_frame = vec![];
                for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                    // offset_for_ref_frame[ i ] 0 se(v)
                    offset_for_ref_frame.push(value.get_signed());
                }

                Self::OnFrameNumbers {
                    delta_pic_order_always_zero,
                    offset_for_non_ref_pic,
                    offset_for_top_to_bottom_field,
                    num_ref_frames_in_pic_order_cnt_cycle,
                    offset_for_ref_frame,
                }
            }
            2 => Self::OnFieldNumbers,
            _ => {
                return Err(SpsDecodeError {
                    kind: SpsDecodeErrorKind::InvalidData,
                    help: "PicOrderCntType",
                })
            }
        })
    }
}

/// frame_mbs_only_flag equal to 0 specifies that coded pictures of the coded
/// video sequence may either be coded fields or coded frames.
/// frame_mbs_only_flag equal to 1 specifies that every coded picture of the
/// coded video sequence is a coded frame containing only frame macroblocks.
/// The allowed range of values for pic_width_in_mbs_minus1,
/// pic_height_in_map_units_minus1, and frame_mbs_only_flag is specified by
/// constraints in Annex A. Depending on frame_mbs_only_flag, semantics are
/// assigned to pic_height_in_map_units_minus1 as follows.
///
/// – If frame_mbs_only_flag is equal to 0, pic_height_in_map_units_minus1 plus
/// 1 is the height of a field in units of macroblocks.
///
/// – Otherwise (frame_mbs_only_flag is equal to 1),
/// pic_height_in_map_units_minus1 plus 1 is the height of a frame in units of
/// macroblocks.
///
/// The variable FrameHeightInMbs is derived as
///
/// FrameHeightInMbs = ( 2 − frame_mbs_only_flag ) * PicHeightInMapUnits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameMbsOnly {
    FrameMode,
    AdaptiveFrameFieldMode {
        /// mb_adaptive_frame_field_flag equal to 0 specifies no switching
        /// between frame and field macroblocks within a
        /// picture. mb_adaptive_frame_field_flag equal to 1 specifies the
        /// possible use of switching between frame and field
        /// macroblocks within frames. When mb_adaptive_frame_field_flag is not
        /// present, it shall be inferred to be equal to 0.
        mb_adaptive_frame_field: bool,
    },
}

/// frame_cropping_flag equal to 1 specifies that the frame cropping offset
/// parameters follow next in the sequence parameter set. frame_cropping_flag
/// equal to 0 specifies that the frame cropping offset parameters are not
/// present.
///
/// frame_crop_left_offset, frame_crop_right_offset, frame_crop_top_offset,
/// frame_crop_bottom_offset specify the samples of the pictures in the coded
/// video sequence that are output from the decoding process, in terms of a
/// rectangular region specified in frame coordinates for output.
///
/// The variables CropUnitX and CropUnitY are derived as follows.
///
/// – If ChromaArrayType is equal to 0, CropUnitX and CropUnitY are derived as:
///
/// CropUnitX = 1
/// CropUnitY = 2 − frame_mbs_only_flag
///
/// – Otherwise (ChromaArrayType is equal to 1, 2, or 3), CropUnitX and
/// CropUnitY are derived as:
///
/// CropUnitX = SubWidthC
/// CropUnitY = SubHeightC * ( 2 − frame_mbs_only_flag )
///
/// The frame cropping rectangle contains luma samples with horizontal frame
/// coordinates from CropUnitX * frame_crop_left_offset to PicWidthInSamplesL −
/// ( CropUnitX * frame_crop_right_offset + 1 ) and vertical frame coordinates
/// from CropUnitY * frame_crop_top_offset to ( 16 * FrameHeightInMbs ) −
/// ( CropUnitY * frame_crop_bottom_offset + 1 ), inclusive. The value of
/// frame_crop_left_offset shall be in the range of 0 to ( PicWidthInSamplesL /
/// CropUnitX ) − ( frame_crop_right_offset + 1 ), inclusive; and the value of
/// frame_crop_top_offset shall be in the range of 0 to ( 16 * FrameHeightInMbs
/// / CropUnitY ) − ( frame_crop_bottom_offset + 1 ), inclusive.
///
/// When frame_cropping_flag is equal to 0, the values of
/// frame_crop_left_offset, frame_crop_right_offset, frame_crop_top_offset, and
/// frame_crop_bottom_offset shall be inferred to be equal to 0.
/// When ChromaArrayType is not equal to 0, the corresponding specified samples
/// of the two chroma arrays are the samples having frame coordinates ( x /
/// SubWidthC, y / SubHeightC ), where ( x, y ) are the frame coordinates of the
/// specified luma samples.
///
/// For decoded fields, the specified samples of the decoded field are the
/// samples that fall within the rectangle specified in frame coordinates.
#[derive(Debug, Clone, Copy)]
pub struct FrameCropping {
    pub left_offset: usize,
    pub right_offset: usize,
    pub top_offset: usize,
    pub bottom_offset: usize,
}

#[derive(Debug, Clone)]
pub struct Sps {
    /// profile_idc and level_idc indicate the profile and level to which the
    /// coded video sequence conforms.
    pub profile_idc: Profile,
    /// `constraint_set0_flag` equal to 1 indicates that the coded video
    /// sequence obeys all constraints specified in subclause A.2.1.
    /// constraint_set0_flag equal to 0 indicates that the coded video sequence
    /// may or may not obey all constraints specified in subclause A.2.1.
    ///
    /// `constraint_set1_flag` equal to 1 indicates that the coded video
    /// sequence obeys all constraints specified in subclause A.2.2.
    /// constraint_set1_flag equal to 0 indicates that the coded video sequence
    /// may or may not obey all constraints specified in subclause A.2.2.
    ///
    /// `constraint_set2_flag` equal to 1 indicates that the coded video
    /// sequence obeys all constraints specified in subclause A.2.3.
    /// constraint_set2_flag equal to 0 indicates that the coded video sequence
    /// may or may not obey all constraints specified in subclause A.2.3.
    /// NOTE 1 – When one or more than one of constraint_set0_flag,
    /// constraint_set1_flag, or constraint_set2_flag are equal to 1, the
    /// coded video sequence must obey the constraints of all of the indicated
    /// subclauses of subclause A.2. When profile_idc is equal to 44, 100,
    /// 110, 122, or 244, the values of constraint_set0_flag,
    /// constraint_set1_flag, and constraint_set2_flag must all be equal
    /// to 0.
    ///
    /// `constraint_set3_flag` is specified as follows.
    /// – If profile_idc is equal to 66, 77, or 88 and level_idc is equal to 11,
    /// constraint_set3_flag equal to 1 indicates that the coded video
    /// sequence obeys all constraints specified in Annex A for level 1b and
    /// constraint_set3_flag equal to 0 indicates that the coded video
    /// sequence may or may not obey all constraints specified in Annex A for
    /// level 1b. – Otherwise, if profile_idc is equal to 100 or 110,
    /// constraint_set3_flag equal to 1 indicates that the coded video
    /// sequence obeys all constraints specified in Annex A for the High 10
    /// Intra profile, and constraint_set3_flag equal to 0 indicates that
    /// the coded video sequence may or may not obey these corresponding
    /// constraints. – Otherwise, if profile_idc is equal to 122,
    /// constraint_set3_flag equal to 1 indicates that the coded video sequence
    /// obeys all constraints specified in Annex A for the High 4:2:2 Intra
    /// profile, and constraint_set3_flag equal to 0 indicates that the
    /// coded video sequence may or may not obey these corresponding
    /// constraints. – Otherwise, if profile_idc is equal to 44,
    /// constraint_set3_flag shall be equal to 1. When profile_idc is equal to
    /// 44, the value of 0 for constraint_set3_flag is forbidden.
    /// – Otherwise, if profile_idc is equal to 244, constraint_set3_flag equal
    /// to 1 indicates that the coded video sequence obeys all constraints
    /// specified in Annex A for the High 4:4:4 Intra profile, and
    /// constraint_set3_flag equal to 0 indicates that the coded video
    /// sequence may or may not obey these corresponding constraints.
    /// – Otherwise (profile_idc is equal to 66, 77, or 88 and level_idc is not
    /// equal to 11), the value of 1 for constraint_set3_flag is reserved
    /// for future use by ITU-T | ISO/IEC. constraint_set3_flag shall be equal
    /// to 0 for coded video sequences with profile_idc equal to 66, 77, or
    /// 88 and level_idc not equal to 11 in bitstreams conforming to this
    /// Recommendation | International Standard. Decoders shall ignore the value
    /// of constraint_set3_flag when profile_idc is equal to 66, 77, or 88
    /// and level_idc is not equal to 11.
    ///
    /// `constraint_set4_flag` has semantics as specified in Annex H. Decoders
    /// conforming to the profiles specified in Annex A and Annex G may
    /// ignore the value of constraint_set4_flag.
    ///
    /// `constraint_set5_flag` has semantics as specified in Annex H. Decoders
    /// conforming to the profiles specified in Annex A and Annex G may
    /// ignore the value of constraint_set5_flag.
    pub constraint_setx_flags: [bool; 6],
    /// profile_idc and level_idc indicate the profile and level to which the
    /// coded video sequence conforms.
    pub level_idc: u32,
    /// seq_parameter_set_id identifies the sequence parameter set that is
    /// referred to by the picture parameter set. The value of
    /// seq_parameter_set_id shall be in the range of 0 to 31, inclusive.
    pub seq_parameter_set_id: usize,
    /// chroma_format_idc specifies the chroma sampling relative to the luma
    /// sampling as specified in subclause 6.2. The value of chroma_format_idc
    /// shall be in the range of 0 to 3, inclusive. When chroma_format_idc is
    /// not present, it shall be inferred to be equal to 1 (4:2:0 chroma
    /// format).
    pub chroma_format_idc: ChromaFormat,
    /// separate_colour_plane_flag equal to 1 specifies that the three colour
    /// components of the 4:4:4 chroma format are coded separately.
    /// separate_colour_plane_flag equal to 0 specifies that the colour
    /// components are not coded separately. When separate_colour_plane_flag
    /// is not present, it shall be inferred to be equal to 0. When
    /// separate_colour_plane_flag is equal to 1, the primary coded picture
    /// consists of three separate components, each of which consists of coded
    /// samples of one colour plane (Y, Cb or Cr) that each use the
    /// monochrome coding syntax. In this case, each colour plane is
    /// associated with a specific colour_plane_id value.
    pub separate_colour_plane: bool,
    /// bit_depth_luma_minus8 specifies the bit depth of the samples of the luma
    /// array and the value of the luma quantisation parameter range offset
    /// QpBdOffsetY, as specified by
    ///
    /// ```BitDepthY = 8 + bit_depth_luma_minus8```
    /// ```QpBdOffsetY = 6 * bit_depth_luma_minus8```
    ///
    /// When bit_depth_luma_minus8 is not present, it shall be inferred to be
    /// equal to 0. bit_depth_luma_minus8 shall be in the range of 0 to 6,
    /// inclusive.
    pub bit_depth_luma_minus8: BitDepth,
    /// bit_depth_chroma_minus8 specifies the bit depth of the samples of the
    /// chroma arrays and the value of the chroma quantisation parameter
    /// range offset QpBdOffsetC, as specified by
    ///
    /// ```BitDepthC = 8 + bit_depth_chroma_minus8```
    /// ```QpBdOffsetC = 6 * bit_depth_chroma_minus8```
    ///
    /// When bit_depth_chroma_minus8 is not present, it shall be inferred to be
    /// equal to 0. bit_depth_chroma_minus8 shall be in the range of 0 to 6,
    /// inclusive.
    ///
    /// The variable RawMbBits is derived as
    ///
    /// ```RawMbBits = 256 * BitDepthY + 2 * MbWidthC * MbHeightC * BitDepthC```
    pub bit_depth_chroma_minus8: BitDepth,
    /// qpprime_y_zero_transform_bypass_flag equal to 1 specifies that, when
    /// QP′Y is equal to 0, a transform bypass operation for the transform
    /// coefficient decoding process and picture construction process prior to
    /// deblocking filter process as specified in subclause 8.5 shall be
    /// applied. qpprime_y_zero_transform_bypass_flag equal to 0 specifies that
    /// the transform coefficient decoding process and picture construction
    /// process prior to deblocking filter process shall not 74
    ///
    /// use the transform bypass operation. When
    /// qpprime_y_zero_transform_bypass_flag is not present, it shall be
    /// inferred to be equal to 0.
    pub qpprime_y_zero_transform_bypass: bool,
    /// seq_scaling_matrix_present_flag equal to 1 specifies that the flags
    /// seq_scaling_list_present_flag[ i ] for i = 0..7 or i = 0..11 are
    /// present. seq_scaling_matrix_present_flag equal to 0 specifies that these
    /// flags are not present and the sequence-level scaling list specified
    /// by Flat_4x4_16 shall be inferred for i = 0..5 and the sequence-level
    /// scaling list specified by Flat_8x8_16 shall be inferred for i =
    /// 6..11. When seq_scaling_matrix_present_flag is not present, it shall
    /// be inferred to be equal to 0.
    ///
    /// The scaling lists Flat_4x4_16 and Flat_8x8_16 are specified as follows:
    ///
    /// Flat_4x4_16[ k ] = 16, with k = 0..15,
    /// Flat_8x8_16[ k ] = 16, with k = 0..63.
    pub seq_scaling_matrix_present_flag: bool,
    /// log2_max_frame_num_minus4 specifies the value of the variable
    /// MaxFrameNum that is used in frame_num related derivations as
    /// follows:
    ///
    /// MaxFrameNum = 2( log2_max_frame_num_minus4 + 4 )
    ///
    /// The value of log2_max_frame_num_minus4 shall be in the range of 0 to 12,
    /// inclusive.
    pub log2_max_frame_num_minus4: usize,
    pub pic_order_cnt: PicOrderCnt,
    /// max_num_ref_frames specifies the maximum number of short-term and
    /// long-term reference frames, complementary reference field pairs, and
    /// non-paired reference fields that may be used by the decoding process for
    /// inter prediction of any picture in the sequence. max_num_ref_frames
    /// also determines the size of the sliding window operation as specified
    /// in subclause 8.2.5.3. The value of max_num_ref_frames shall be in the
    /// range of 0 to MaxDpbFrames (as specified in subclause A.3.1 or
    /// A.3.2), inclusive.
    pub max_num_ref_frames: usize,
    /// gaps_in_frame_num_value_allowed_flag specifies the allowed values of
    /// frame_num as specified in subclause 7.4.3 and the decoding process
    /// in case of an inferred gap between values of frame_num as specified in
    /// subclause 8.2.5.2.
    pub gaps_in_frame_num_value_allowed: bool,
    /// pic_width_in_mbs_minus1 plus 1 specifies the width of each decoded
    /// picture in units of macroblocks. The variable for the picture width
    /// in units of macroblocks is derived as
    ///
    /// PicWidthInMbs = pic_width_in_mbs_minus1 + 1
    ///
    /// The variable for picture width for the luma component is derived as
    ///
    /// PicWidthInSamplesL = PicWidthInMbs * 16
    ///
    /// The variable for picture width for the chroma components is derived as
    ///
    /// PicWidthInSamplesC = PicWidthInMbs * MbWidthC
    pub pic_width_in_mbs_minus1: usize,
    /// pic_height_in_map_units_minus1 plus 1 specifies the height in slice
    /// group map units of a decoded frame or field. The variables
    /// PicHeightInMapUnits and PicSizeInMapUnits are derived as
    ///
    /// PicHeightInMapUnits = pic_height_in_map_units_minus1 + 1
    /// PicSizeInMapUnits = PicWidthInMbs * PicHeightInMapUnits
    pub pic_height_in_map_units_minus1: usize,
    pub frame_mbs_only: FrameMbsOnly,
    /// direct_8x8_inference_flag specifies the method used in the derivation
    /// process for luma motion vectors for B_Skip, B_Direct_16x16 and
    /// B_Direct_8x8 as specified in subclause 8.4.1.2. When frame_mbs_only_flag
    /// is equal to 0, direct_8x8_inference_flag shall be equal to 1.
    pub direct_8x8_inference: bool,
    pub frame_cropping: Option<FrameCropping>,
    pub vui: Option<Vui>,
}

impl Sps {
    fn read_scaling_list(
        bits: &mut Bits,
        scaling_list: &mut [u32],
        use_default_scaling_matrix_flag: &mut bool,
    ) {
        let mut last_scale = 8;
        let mut get_scale = 0;

        for i in 0..scaling_list.len() {
            if get_scale != 0 {
                // delta_scale 0 | 1 se(v)
                let delta_scale = bits.get_signed();
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
}

impl TryFrom<&[u8]> for Sps {
    type Error = SpsDecodeError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut bits = Bits::new(value, 0);

        // profile_idc 0 u(8)
        let profile_idc = Profile::try_from(bits.get_bits(8) as u8)?;
        let constraint_setx_flags = [
            // constraint_set0_flag 0 u(1)
            bits.get_bit(),
            // constraint_set1_flag 0 u(1)
            bits.get_bit(),
            // constraint_set2_flag 0 u(1)
            bits.get_bit(),
            // constraint_set3_flag 0 u(1)
            bits.get_bit(),
            // constraint_set4_flag 0 u(1)
            bits.get_bit(),
            // constraint_set5_flag 0 u(1)
            bits.get_bit(),
        ];

        // reserved_zero_2bits shall be equal to 0. Other values of reserved_zero_2bits
        // may be specified in the future by ITU-T | ISO/IEC. Decoders shall ignore the
        // value of reserved_zero_2bits.
        bits.get_bits(2);

        // level_idc 0 u(8)
        let level_idc = bits.get_bits(8);

        // seq_parameter_set_id 0 ue(v)
        let seq_parameter_set_id = bits.get_unsigned();

        // chroma_format_idc 0 ue(v)
        let chroma_format_idc = ChromaFormat::try_from(bits.get_unsigned() as u8)?;

        // separate_colour_plane_flag 0 u(1)
        let separate_colour_plane = if chroma_format_idc == ChromaFormat::Yuv444 {
            bits.get_bit()
        } else {
            false
        };

        // bit_depth_luma_minus8 0 ue(v)
        let bit_depth_luma_minus8 = BitDepth::try_from(bits.get_unsigned() as u8)?;

        // bit_depth_chroma_minus8 0 ue(v)
        let bit_depth_chroma_minus8 = BitDepth::try_from(bits.get_unsigned() as u8)?;

        // qpprime_y_zero_transform_bypass_flag 0 u(1)
        let qpprime_y_zero_transform_bypass = bits.get_bit();

        // seq_scaling_matrix_present_flag 0 u(1)
        let seq_scaling_matrix_present_flag = bits.get_bit();

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
                // seq_scaling_list_present_flag[ i ] 0 u(1)
                seq_scaling_list_present_flag[i] = bits.get_bit();
                if seq_scaling_list_present_flag[i] {
                    if i < 6 {
                        Self::read_scaling_list(
                            &mut bits,
                            &mut scaling_list_4x4[i],
                            &mut use_default_scaling_matrix_flag_4x4[i],
                        );
                    } else {
                        Self::read_scaling_list(
                            &mut bits,
                            &mut scaling_list_8x8[i - 6],
                            &mut use_default_scaling_matrix_flag_8x8[i],
                        )
                    }
                }
            }
        }

        // log2_max_frame_num_minus4 0 ue(v)
        let log2_max_frame_num_minus4 = bits.get_unsigned();

        // pic_order_cnt_type 0 ue(v)
        let pic_order_cnt = PicOrderCnt::try_from(&mut bits)?;

        // max_num_ref_frames 0 ue(v)
        let max_num_ref_frames = bits.get_unsigned();

        // gaps_in_frame_num_value_allowed_flag 0 u(1)
        let gaps_in_frame_num_value_allowed = bits.get_bit();

        // pic_width_in_mbs_minus1 0 ue(v)
        let pic_width_in_mbs_minus1 = bits.get_unsigned();

        // pic_height_in_map_units_minus1 0 ue(v)
        let pic_height_in_map_units_minus1 = bits.get_unsigned();

        // frame_mbs_only_flag 0 u(1)
        let frame_mbs_only_flag = bits.get_bit();
        let frame_mbs_only = if !frame_mbs_only_flag {
            FrameMbsOnly::AdaptiveFrameFieldMode {
                // mb_adaptive_frame_field_flag 0 u(1)
                mb_adaptive_frame_field: bits.get_bit(),
            }
        } else {
            FrameMbsOnly::FrameMode
        };

        // direct_8x8_inference_flag 0 u(1)
        let direct_8x8_inference = bits.get_bit();

        // frame_cropping_flag 0 u(1)
        let frame_cropping_flag = bits.get_bit();
        let frame_cropping = if frame_cropping_flag {
            Some(FrameCropping {
                // frame_crop_left_offset 0 ue(v)
                left_offset: bits.get_unsigned(),
                // frame_crop_right_offset 0 ue(v)
                right_offset: bits.get_unsigned(),
                // frame_crop_top_offset 0 ue(v)
                top_offset: bits.get_unsigned(),
                // frame_crop_bottom_offset 0 ue(v)
                bottom_offset: bits.get_unsigned(),
            })
        } else {
            None
        };

        // vui_parameters_present_flag 0 u(1)
        let vui_parameters_present_flag = bits.get_bit();
        let vui = if vui_parameters_present_flag {
            Some(Vui::try_from(&mut bits)?)
        } else {
            None
        };

        Ok(Self {
            profile_idc,
            constraint_setx_flags,
            level_idc,
            seq_parameter_set_id,
            chroma_format_idc,
            separate_colour_plane,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass,
            seq_scaling_matrix_present_flag,
            log2_max_frame_num_minus4,
            pic_order_cnt,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed,
            pic_height_in_map_units_minus1,
            pic_width_in_mbs_minus1,
            frame_mbs_only,
            direct_8x8_inference,
            frame_cropping,
            vui,
        })
    }
}
