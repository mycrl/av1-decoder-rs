use crate::{
    bitstream::{BitRead, Bits, ExpGolomb},
    Session,
};

use super::sps::{FrameMbsOnly, PicOrderCnt};

#[derive(Debug)]
pub enum SliceDecodeErrorKind {
    UnSupports,
    SpsNotFound,
    PpsNotFound,
}

#[derive(Debug)]
pub struct SliceDecodeError {
    pub kind: SliceDecodeErrorKind,
    pub help: &'static str,
}

impl std::error::Error for SliceDecodeError {}

impl std::fmt::Display for SliceDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceType {
    I,
    P,
    B,
    Sp,
    Si,
}

impl TryFrom<u8> for SliceType {
    type Error = SliceDecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 | 5 => Self::I,
            1 | 6 => Self::P,
            2 | 7 => Self::B,
            3 | 8 => Self::Sp,
            4 | 9 => Self::Si,
            _ => {
                return Err(SliceDecodeError {
                    kind: SliceDecodeErrorKind::UnSupports,
                    help: "SliceType",
                })
            }
        })
    }
}

pub struct SliceHeader {
    pub first_mb_in_slice: usize,
    pub slice_type: SliceType,
    pub colour_plane_id: Option<u8>,
    pub frame_num: usize,
    pub field_pic: bool,
    pub bottom_field: bool,
    pub idr_pic_id: Option<usize>,
    pub pic_order_cnt_lsb: Option<usize>,
    pub delta_pic_order_cnt_bottom: Option<isize>,
    pub delta_pic_order_cnt: (Option<isize>, Option<isize>),
    pub redundant_pic_cnt: Option<usize>,
    pub direct_spatial_mv_pred: Option<bool>,
}

impl SliceHeader {
    pub fn decode(session: &Session, value: &mut Bits) -> Result<Self, SliceDecodeError> {
        // first_mb_in_slice 2 ue(v)
        let first_mb_in_slice = value.get_unsigned();

        // slice_type 2 ue(v)
        let slice_type = SliceType::try_from(value.get_unsigned() as u8)?;

        // pic_parameter_set_id 2 ue(v)
        let pic_parameter_set_id = value.get_unsigned();

        let pps = session
            .ppss
            .get(&pic_parameter_set_id)
            .ok_or(SliceDecodeError {
                kind: SliceDecodeErrorKind::PpsNotFound,
                help: "",
            })?;

        let sps = session
            .spss
            .get(&pps.seq_parameter_set_id)
            .ok_or(SliceDecodeError {
                kind: SliceDecodeErrorKind::SpsNotFound,
                help: "",
            })?;

        let colour_plane_id = if sps.separate_colour_plane {
            // colour_plane_id 2 u(2)
            Some(value.get_bits(2) as u8)
        } else {
            None
        };

        // frame_num 2 u(v)
        let frame_num_bits = sps.log2_max_frame_num_minus4 + 4;
        let frame_num = value.get_bits(frame_num_bits) as usize;

        let mut field_pic = false;
        let mut bottom_field = false;
        if sps.frame_mbs_only != FrameMbsOnly::FrameMode {
            // field_pic_flag 2 u(1)
            field_pic = value.get_bit();
            if field_pic {
                // bottom_field_flag 2 u(1)
                bottom_field = value.get_bit();
            }
        }

        let idr_pic_id = if slice_type == SliceType::I {
            // idr_pic_id 2 ue(v)
            Some(value.get_unsigned())
        } else {
            None
        };

        let mut pic_order_cnt_lsb = None;
        let mut delta_pic_order_cnt_bottom = None;
        let mut delta_pic_order_cnt = (None, None);
        match &sps.pic_order_cnt {
            PicOrderCnt::None {
                log2_max_pic_order_cnt_lsb_minus4,
            } => {
                // pic_order_cnt_lsb 2 u(v)
                let pic_order_cnt_lsb_bits = log2_max_pic_order_cnt_lsb_minus4 + 4;
                pic_order_cnt_lsb = Some(value.get_bits(pic_order_cnt_lsb_bits) as usize);
                if pps.bottom_field_pic_order_in_frame_present && !field_pic {
                    // delta_pic_order_cnt_bottom 2 se(v)
                    delta_pic_order_cnt_bottom = Some(value.get_signed());
                }
            }
            PicOrderCnt::OnFrameNumbers {
                delta_pic_order_always_zero,
                ..
            } => {
                if *delta_pic_order_always_zero {
                    // delta_pic_order_cnt[ 0 ] 2 se(v)
                    delta_pic_order_cnt.0 = Some(value.get_signed());
                    if pps.bottom_field_pic_order_in_frame_present && !field_pic {
                        // delta_pic_order_cnt[ 1 ] 2 se(v)
                        delta_pic_order_cnt.1 = Some(value.get_signed())
                    }
                }
            }
            _ => (),
        };

        let redundant_pic_cnt = if pps.redundant_pic_cnt_present {
            // redundant_pic_cnt 2 ue(v)
            Some(value.get_unsigned())
        } else {
            None
        };

        // let direct_spatial_mv_pred = 

        Ok(Self {
            first_mb_in_slice,
            slice_type,
            colour_plane_id,
            frame_num,
            field_pic,
            bottom_field,
            idr_pic_id,
            pic_order_cnt_lsb,
            delta_pic_order_cnt_bottom,
            delta_pic_order_cnt,
            redundant_pic_cnt,
        })
    }
}
