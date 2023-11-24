use crate::bitstream::{BitRead, Bits, ExpGolomb};

#[derive(Debug)]
pub enum PpsDecodeErrorKind {
    InvalidData,
}

#[derive(Debug)]
pub struct PpsDecodeError {
    pub kind: PpsDecodeErrorKind,
    pub help: &'static str,
}

impl std::error::Error for PpsDecodeError {}

impl std::fmt::Display for PpsDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// top_left[ i ] and bottom_right[ i ] specify the top-left and bottom-right
/// corners of a rectangle, respectively. top_left[ i ] and bottom_right[ i ]
/// are slice group map unit positions in a raster scan of the picture for the
/// slice group map units. For each rectangle i, all of the following
/// constraints shall be obeyed by the values of the syntax elements top_left[ i
/// ] and bottom_right[ i ]:
///
/// – top_left[ i ] shall be less than or equal to bottom_right[ i ] and
/// bottom_right[ i ] shall be less than PicSizeInMapUnits.
///
/// – ( top_left[ i ] % PicWidthInMbs ) shall be less than or equal to the value
/// of ( bottom_right[ i ] % PicWidthInMbs ).
#[derive(Debug, Clone)]
pub struct SliceGroupMapTypeBoxOut {
    pub top_left: usize,
    pub bottom_right: usize,
}

/// slice_group_map_type specifies how the mapping of slice group map units to
/// slice groups is coded. The value of slice_group_map_type shall be in the
/// range of 0 to 6, inclusive.
///
/// slice_group_map_type equal to 0 specifies interleaved slice groups.
///
/// slice_group_map_type equal to 1 specifies a dispersed slice group mapping.
///
/// slice_group_map_type equal to 2 specifies one or more "foreground" slice
/// groups and a "leftover" slice group. slice_group_map_type values equal to 3,
/// 4, and 5 specify changing slice groups. When num_slice_groups_minus1 is
/// not equal to 1, slice_group_map_type shall not be equal to 3, 4, or 5.
/// slice_group_map_type equal to 6 specifies an explicit assignment of a slice
/// group to each slice group map unit. Slice group map units are specified as
/// follows.
///
/// – If frame_mbs_only_flag is equal to 0 and mb_adaptive_frame_field_flag is
/// equal to 1 and the coded picture is a frame, the slice group map units are
/// macroblock pair units.
///
/// – Otherwise, if frame_mbs_only_flag is equal to 1 or the coded picture is a
/// field, the slice group map units are units of macroblocks.
///
/// – Otherwise (frame_mbs_only_flag is equal to 0 and
/// mb_adaptive_frame_field_flag is equal to 0 and the coded picture is a
/// frame), the slice group map units are units of two macroblocks that are
/// vertically contiguous as in a frame macroblock pair of an MBAFF frame.
#[derive(Debug, Clone)]
pub enum SliceGroupMapType {
    InterleavedSliceGroup {
        /// run_length_minus1[ i ] is used to specify the number of consecutive
        /// slice group map units to be assigned to the i-th slice group
        /// in raster scan order of slice group map units. The value of
        /// run_length_minus1[ i ] shall be in the range of 0
        /// to PicSizeInMapUnits − 1, inclusive.
        run_length_minus1: Vec<usize>,
    },
    ForegroundWithLeftOver,
    BoxOut(Vec<SliceGroupMapTypeBoxOut>),
    RasterScan {
        /// slice_group_change_direction_flag is used with slice_group_map_type
        /// to specify the refined map type when slice_group_map_type is
        /// 3, 4, or 5.
        slice_group_change_direction: bool,
        /// slice_group_change_rate_minus1 is used to specify the variable
        /// SliceGroupChangeRate. SliceGroupChangeRate specifies the
        /// multiple in number of slice group map units by which the size of a
        /// slice group can change from one picture to the next. The
        /// value of slice_group_change_rate_minus1 shall be in the range of 0
        /// to PicSizeInMapUnits − 1, inclusive. The
        /// SliceGroupChangeRate variable is specified as follows:
        ///
        /// SliceGroupChangeRate = slice_group_change_rate_minus1 + 1
        slice_group_change_rate_minus1: usize,
    },
    ExplicitAssignment {
        /// pic_size_in_map_units_minus1 is used to specify the number of slice
        /// group map units in the picture. pic_size_in_map_units_minus1
        /// shall be equal to PicSizeInMapUnits − 1.
        pic_size_in_map_units_minus1: usize,
        /// slice_group_id[ i ] identifies a slice group of the i-th slice group
        /// map unit in raster scan order. The length of the
        /// slice_group_id[ i ] syntax element is Ceil( Log2(
        /// num_slice_groups_minus1 + 1 ) ) bits. The value of slice_group_id[ i
        /// ] shall be in the range of 0 to num_slice_groups_minus1,
        /// inclusive.
        slice_group_ids: Vec<u32>,
    },
}

impl SliceGroupMapType {
    fn int_log2(x: usize) -> usize {
        let mut log = 0;
        while (x >> log) > 0 {
            log += 1;
        }

        if log > 0 && x == 1 << (log - 1) {
            log -= 1;
        }

        log
    }

    fn parse(value: &mut Bits, num_slice_groups_minus1: usize) -> Result<Self, PpsDecodeError> {
        // slice_group_map_type 1 ue(v)
        Ok(match value.get_unsigned() as u8 {
            0 => {
                let mut run_length_minus1 = Vec::with_capacity(num_slice_groups_minus1);
                for _ in 0..num_slice_groups_minus1 {
                    // run_length_minus1[ iGroup ] 1 ue(v)
                    run_length_minus1.push(value.get_unsigned());
                }

                Self::InterleavedSliceGroup { run_length_minus1 }
            }
            1 => Self::ForegroundWithLeftOver,
            2 => {
                let mut box_out = Vec::with_capacity(num_slice_groups_minus1);
                for _ in 0..num_slice_groups_minus1 {
                    box_out.push(SliceGroupMapTypeBoxOut {
                        // top_left[ iGroup ] 1 ue(v)
                        top_left: value.get_unsigned(),
                        // bottom_right[ iGroup ] 1 ue(v)
                        bottom_right: value.get_unsigned(),
                    });
                }

                Self::BoxOut(box_out)
            }
            3 | 4 | 5 => Self::RasterScan {
                // slice_group_change_direction_flag 1 u(1)
                slice_group_change_direction: value.get_bit(),
                // slice_group_change_rate_minus1 1 ue(v)
                slice_group_change_rate_minus1: value.get_unsigned(),
            },
            6 => {
                // pic_size_in_map_units_minus1 1 ue(v)
                let pic_size_in_map_units_minus1 = value.get_unsigned();
                let mut slice_group_ids = Vec::with_capacity(pic_size_in_map_units_minus1);
                let int_log2 = Self::int_log2(num_slice_groups_minus1);
                for _ in 0..pic_size_in_map_units_minus1 {
                    // slice_group_id[ i ] 1 u(v)
                    slice_group_ids.push(value.get_bits(int_log2));
                }

                Self::ExplicitAssignment {
                    pic_size_in_map_units_minus1,
                    slice_group_ids,
                }
            }
            _ => {
                return Err(PpsDecodeError {
                    kind: PpsDecodeErrorKind::InvalidData,
                    help: "SliceGroupMapType",
                })
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Pps {
    /// pic_parameter_set_id identifies the picture parameter set that is
    /// referred to in the slice header. The value of pic_parameter_set_id
    /// shall be in the range of 0 to 255, inclusive.
    pub pic_parameter_set_id: u8,
    /// seq_parameter_set_id refers to the active sequence parameter set. The
    /// value of seq_parameter_set_id shall be in the range of 0 to 31,
    /// inclusive.
    pub seq_parameter_set_id: u8,
    /// entropy_coding_mode_flag selects the entropy decoding method to be
    /// applied for the syntax elements for which two descriptors appear in
    /// the syntax tables as follows.
    ///
    /// – If entropy_coding_mode_flag is equal to 0, the method specified by the
    /// left descriptor in the syntax table is applied (Exp-Golomb coded,
    /// see subclause 9.1 or CAVLC, see subclause 9.2).
    ///
    /// – Otherwise (entropy_coding_mode_flag is equal to 1), the method
    /// specified by the right descriptor in the syntax table is applied
    /// (CABAC, see subclause 9.3).
    pub entropy_coding_mode: bool,
    /// bottom_field_pic_order_in_frame_present_flag equal to 1 specifies that
    /// the syntax elements delta_pic_order_cnt_bottom (when
    /// pic_order_cnt_type is equal to 0) or delta_pic_order_cnt[ 1 ] (when
    /// pic_order_cnt_type is equal to 1), which are related to picture order
    /// counts for the bottom field of a coded frame, are present in the
    /// slice headers for coded frames as specified in subclause 7.3.3.
    /// bottom_field_pic_order_in_frame_present_flag equal to 0 specifies that
    /// the syntax elements delta_pic_order_cnt_bottom and
    /// delta_pic_order_cnt[ 1 ] are not present in the slice headers.
    pub bottom_field_pic_order_in_frame_present: bool,
    /// num_slice_groups_minus1 plus 1 specifies the number of slice groups for
    /// a picture. When num_slice_groups_minus1 is equal to 0, all slices of
    /// the picture belong to the same slice group. The allowed range of
    /// num_slice_groups_minus1 is specified in Annex A.
    pub num_slice_groups_minus1: usize,
    pub slice_group_map_type: Option<SliceGroupMapType>,
    /// num_ref_idx_l0_default_active_minus1 specifies how
    /// num_ref_idx_l0_active_minus1 is inferred for P, SP, and B
    /// slices with num_ref_idx_active_override_flag equal to 0. The value of
    /// num_ref_idx_l0_default_active_minus1 shall be in the range of 0 to
    /// 31, inclusive.
    pub num_ref_idx_l0_default_active_minus1: usize,
    /// num_ref_idx_l1_default_active_minus1 specifies how
    /// num_ref_idx_l1_active_minus1 is inferred for B slices with
    /// num_ref_idx_active_override_flag equal to 0. The value of
    /// num_ref_idx_l1_default_active_minus1 shall be in the range of 0 to
    /// 31, inclusive.
    pub num_ref_idx_l1_default_active_minus1: usize,
    /// weighted_pred_flag equal to 0 specifies that the default weighted
    /// prediction shall be applied to P and SP slices. weighted_pred_flag
    /// equal to 1 specifies that explicit weighted prediction shall be applied
    /// to P and SP slices.
    pub weighted_pred: bool,
    /// weighted_bipred_idc equal to 0 specifies that the default weighted
    /// prediction shall be applied to B slices. weighted_bipred_idc equal
    /// to 1 specifies that explicit weighted prediction shall be applied to B
    /// slices. weighted_bipred_idc equal to 2 specifies that implicit
    /// weighted prediction shall be applied to B slices. The value of
    /// weighted_bipred_idc shall be in the range of 0 to 2, inclusive.
    pub weighted_bipred_idc: u8,
    /// pic_init_qp_minus26 specifies the initial value minus 26 of SliceQPY for
    /// each slice. The initial value is modified at the slice layer when a
    /// non-zero value of slice_qp_delta is decoded, and is modified further
    /// when a non-zero value of mb_qp_delta is decoded at the macroblock
    /// layer. The value of pic_init_qp_minus26 shall be in the range of
    /// −(26 + QpBdOffsetY ) to +25, inclusive.
    pub pic_init_qp_minus26: isize,
    /// pic_init_qs_minus26 specifies the initial value minus 26 of SliceQSY for
    /// all macroblocks in SP or SI slices. The initial value is modified at
    /// the slice layer when a non-zero value of slice_qs_delta is decoded. The
    /// value of pic_init_qs_minus26 shall be in the range of −26 to +25,
    /// inclusive.
    pub pic_init_qs_minus26: isize,
    /// chroma_qp_index_offset specifies the offset that shall be added to QPY
    /// and QSY for addressing the table of QPC values for the Cb chroma
    /// component. The value of chroma_qp_index_offset shall be in the range of
    /// −12 to +12, inclusive.
    pub chroma_qp_index_offset: isize,
    /// deblocking_filter_control_present_flag equal to 1 specifies that a set
    /// of syntax elements controlling the characteristics of the deblocking
    /// filter is present in the slice header.
    /// deblocking_filter_control_present_flag equal to 0 specifies that the
    /// set of syntax elements controlling the characteristics of the deblocking
    /// filter is not present in the slice headers and their inferred values
    /// are in effect.
    pub deblocking_filter_control_present: bool,
    /// constrained_intra_pred_flag equal to 0 specifies that intra prediction
    /// allows usage of residual data and decoded samples of neighbouring
    /// macroblocks coded using Inter macroblock prediction modes for the
    /// prediction of macroblocks coded using Intra macroblock prediction
    /// modes. constrained_intra_pred_flag equal to 1 specifies onstrained
    /// intra prediction, in which case prediction of macroblocks coded using
    /// Intra macroblock prediction modes only uses residual data and
    /// decoded samples from I or SI macroblock types.
    pub constrained_intra_pred: bool,
    /// redundant_pic_cnt_present_flag equal to 0 specifies that the
    /// redundant_pic_cnt syntax element is not present in slice
    /// headers, coded slice data partition B NAL units, and coded slice data
    /// partition C NAL units that refer (either directly or by association
    /// with a corresponding coded slice data partition A NAL unit) to the
    /// picture parameter set.
    ///
    /// redundant_pic_cnt_present_flag equal to 1 specifies that the
    /// redundant_pic_cnt syntax element is present in all slice
    /// headers, coded slice data partition B NAL units, and coded slice data
    /// partition C NAL units that refer (either directly or by association
    /// with a corresponding coded slice data partition A NAL unit) to the
    /// picture parameter set.
    pub redundant_pic_cnt_present: bool,
}

impl TryFrom<&mut Bits<'_>> for Pps {
    type Error = PpsDecodeError;

    fn try_from(value: &mut Bits<'_>) -> Result<Self, Self::Error> {
        // pic_parameter_set_id 1 ue(v)
        let pic_parameter_set_id = value.get_unsigned() as u8;

        // seq_parameter_set_id 1 ue(v)
        let seq_parameter_set_id = value.get_unsigned() as u8;

        // entropy_coding_mode_flag 1 u(1)
        let entropy_coding_mode = value.get_bit();

        // bottom_field_pic_order_in_frame_present_flag 1 u(1)
        let bottom_field_pic_order_in_frame_present = value.get_bit();

        // num_slice_groups_minus1 1 ue(v)
        let num_slice_groups_minus1 = value.get_unsigned();
        let slice_group_map_type = if num_slice_groups_minus1 > 0 {
            Some(SliceGroupMapType::parse(
                value.as_mut(),
                num_slice_groups_minus1,
            )?)
        } else {
            None
        };

        // num_ref_idx_l0_default_active_minus1 1 ue(v)
        let num_ref_idx_l0_default_active_minus1 = value.get_unsigned();

        // num_ref_idx_l1_default_active_minus1 1 ue(v)
        let num_ref_idx_l1_default_active_minus1 = value.get_unsigned();

        // weighted_pred_flag 1 u(1)
        let weighted_pred = value.get_bit();

        // weighted_bipred_idc 1 u(2)
        let weighted_bipred_idc = value.get_bits(2) as u8;

        // pic_init_qp_minus26 /* relative to 26 */ 1 se(v)
        let pic_init_qp_minus26 = value.get_signed();

        // pic_init_qs_minus26 /* relative to 26 */ 1 se(v)
        let pic_init_qs_minus26 = value.get_signed();

        // chroma_qp_index_offset 1 se(v)
        let chroma_qp_index_offset = value.get_signed();

        // deblocking_filter_control_present_flag 1 u(1)
        let deblocking_filter_control_present = value.get_bit();

        // constrained_intra_pred_flag 1 u(1)
        let constrained_intra_pred = value.get_bit();

        // redundant_pic_cnt_present_flag 1 u(1)
        let redundant_pic_cnt_present = value.get_bit();

        Ok(Self {
            pic_parameter_set_id,
            seq_parameter_set_id,
            entropy_coding_mode,
            bottom_field_pic_order_in_frame_present,
            num_slice_groups_minus1,
            slice_group_map_type,
            num_ref_idx_l0_default_active_minus1,
            num_ref_idx_l1_default_active_minus1,
            weighted_pred,
            weighted_bipred_idc,
            pic_init_qp_minus26,
            pic_init_qs_minus26,
            chroma_qp_index_offset,
            deblocking_filter_control_present,
            constrained_intra_pred,
            redundant_pic_cnt_present,
        })
    }
}
