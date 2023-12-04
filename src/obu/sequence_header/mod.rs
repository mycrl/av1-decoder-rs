pub mod color_config;

use crate::{
    constants::{SELECT_INTEGER_MV, SELECT_SCREEN_CONTENT_TOOLS},
    util::EasyAtomic,
    Av1DecodeError, Av1DecodeUnknownError, Av1DecoderContext, Buffer,
};

use self::color_config::ColorConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceProfile {
    Main,
    High,
    Professional,
}

impl TryFrom<u8> for SequenceProfile {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Main,
            1 => Self::High,
            2 => Self::Professional,
            _ => return Err(Av1DecodeError::Unknown(Av1DecodeUnknownError::Profile)),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EqualPictureInterval {
    pub num_ticks_per_picture: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct TimingInfo {
    pub num_units_in_display_tick: u32,
    pub time_scale: u32,
    pub equal_picture_interval: Option<EqualPictureInterval>,
}

impl TimingInfo {
    pub fn decode(buf: &mut Buffer<'_>) -> Self {
        // num_units_in_display_tick f(32)
        let num_units_in_display_tick = buf.get_bits(32);

        // time_scale f(32)
        let time_scale = buf.get_bits(32);

        // equal_picture_interval f(1)
        let equal_picture_interval = if buf.get_bit() {
            Some(EqualPictureInterval {
                // num_ticks_per_picture_minus_1 uvlc()
                num_ticks_per_picture: buf.get_uvlc() + 1,
            })
        } else {
            None
        };

        Self {
            num_units_in_display_tick,
            time_scale,
            equal_picture_interval,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DecoderModelInfo {
    pub buffer_delay_length: u8,
    pub num_units_in_decoding_tick: u32,
    pub buffer_removal_time_length: u8,
    pub frame_presentation_time_length: u8,
}

impl DecoderModelInfo {
    pub fn decode(buf: &mut Buffer<'_>) -> Self {
        Self {
            // buffer_delay_length_minus_1 f(5)
            buffer_delay_length: buf.get_bits(5) as u8 + 1,
            // num_units_in_decoding_tick f(32)
            num_units_in_decoding_tick: buf.get_bits(32),
            // buffer_removal_time_length_minus_1 f(5)
            buffer_removal_time_length: buf.get_bits(5) as u8 + 1,
            // frame_presentation_time_length_minus_1 f(5)
            frame_presentation_time_length: buf.get_bits(5) as u8 + 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OperatingParametersInfo {
    pub decoder_buffer_delay: u32,
    pub encoder_buffer_delay: u32,
    pub low_delay_mode_flag: bool,
}

impl OperatingParametersInfo {
    pub fn decode(buf: &mut Buffer<'_>, decoder_model_info: &DecoderModelInfo) -> Self {
        let size = decoder_model_info.buffer_delay_length as usize;
        Self {
            // decoder_buffer_delay[ op ]	f(n)
            decoder_buffer_delay: buf.get_bits(size),
            // encoder_buffer_delay[ op ]	f(n)
            encoder_buffer_delay: buf.get_bits(size),
            // low_delay_mode_flag[ op ]	f(1)
            low_delay_mode_flag: buf.get_bit(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperatingPoint {
    pub idc: u16,
    pub level_idx: u8,
    pub tier: bool,
    pub operating_parameters_info: Option<OperatingParametersInfo>,
    pub initial_display_delay: u8,
}

#[derive(Debug, Clone)]
pub struct FrameIdNumbersPresent {
    pub delta_frame_id_length: u8,
    pub additional_frame_id_length: u8,
}

impl FrameIdNumbersPresent {
    pub fn decode(buf: &mut Buffer<'_>) -> Self {
        Self {
            // delta_frame_id_length_minus_2	f(4)
            delta_frame_id_length: buf.get_bits(4) as u8,
            // additional_frame_id_length_minus_1	f(3)
            additional_frame_id_length: buf.get_bits(4) as u8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SequenceHeaderObu {
    pub seq_profile: SequenceProfile,
    pub still_picture: bool,
    pub reduced_still_picture_header: bool,
    pub timing_info: Option<TimingInfo>,
    pub decoder_model_info: Option<DecoderModelInfo>,
    pub initial_display_delay_present_flag: bool,
    pub operating_points: Vec<OperatingPoint>,
    pub frame_width_bits: u8,
    pub frame_height_bits: u8,
    pub max_frame_width: u16,
    pub max_frame_height: u16,
    pub frame_id_numbers_present: Option<FrameIdNumbersPresent>,
    pub use_128x128_superblock: bool,
    pub enable_filter_intra: bool,
    pub enable_intra_edge_filter: bool,
    pub enable_interintra_compound: bool,
    pub enable_masked_compound: bool,
    pub enable_warped_motion: bool,
    pub enable_dual_filter: bool,
    pub enable_order_hint: bool,
    pub enable_jnt_comp: bool,
    pub enable_ref_frame_mvs: bool,
    pub seq_force_screen_content_tools: u8,
    pub seq_force_integer_mv: u8,
    pub enable_superres: bool,
    pub enable_cdef: bool,
    pub enable_restoration: bool,
    pub color_config: ColorConfig,
    pub film_grain_params_present: bool,
}

impl SequenceHeaderObu {
    pub fn decode(ctx: &Av1DecoderContext, buf: &mut Buffer) -> Result<Self, Av1DecodeError> {
        // seq_profile f(3)
        let seq_profile = SequenceProfile::try_from(buf.get_bits(3) as u8)?;

        // still_picture f(1)
        let still_picture = buf.get_bit();

        // reduced_still_picture_header f(1)
        let reduced_still_picture_header = buf.get_bit();

        let mut timing_info = None;
        let mut decoder_model_info_present_flag = false;
        let mut decoder_model_info = None;
        let mut initial_display_delay_present_flag = false;
        let mut operating_points = Vec::with_capacity(32);

        if reduced_still_picture_header {
            operating_points.push(OperatingPoint {
                idc: 0,
                // seq_level_idx[ 0 ] f(5)
                level_idx: buf.get_bits(5) as u8,
                tier: false,
                operating_parameters_info: None,
                initial_display_delay: 10,
            });
        } else {
            // timing_info_present_flag f(1)
            let timing_info_present_flag = buf.get_bit();
            if timing_info_present_flag {
                timing_info = Some(TimingInfo::decode(buf.as_mut()));

                // decoder_model_info_present_flag f(1)
                decoder_model_info_present_flag = buf.get_bit();
                if decoder_model_info_present_flag {
                    decoder_model_info = Some(DecoderModelInfo::decode(buf.as_mut()));
                }
            }

            // initial_display_delay_present_flag	f(1)
            initial_display_delay_present_flag = buf.get_bit();

            // operating_points_cnt_minus_1	f(5)
            let operating_points_cnt = buf.get_bits(5) as u8 + 1;
            for _ in 0..operating_points_cnt as usize {
                // operating_point_idc[ i ]	f(12)
                let idc = buf.get_bits(12) as u16;

                // seq_level_idx[ i ]	f(5)
                let level_idx = buf.get_bits(5) as u8;
                let tier = if level_idx > 7 {
                    // seq_tier[ i ]	f(1)
                    buf.get_bit()
                } else {
                    false
                };

                let mut operating_parameters_info = None;
                if decoder_model_info_present_flag {
                    // decoder_model_present_for_this_op[ i ]	f(1)
                    let ecoder_model_present = buf.get_bit();
                    if ecoder_model_present {
                        operating_parameters_info = Some(OperatingParametersInfo::decode(
                            buf.as_mut(),
                            &decoder_model_info.unwrap(),
                        ));
                    }
                }

                let initial_display_delay = if initial_display_delay_present_flag {
                    // initial_display_delay_present_for_this_op[ i ]	f(1)
                    let initial_display_delay_present = buf.get_bit();
                    if initial_display_delay_present {
                        // initial_display_delay_minus_1[ i ]	f(4)
                        buf.get_bits(4) as u8 + 1
                    } else {
                        10
                    }
                } else {
                    10
                };

                operating_points.push(OperatingPoint {
                    idc,
                    level_idx,
                    tier,
                    operating_parameters_info,
                    initial_display_delay,
                });
            }
        }

        let ctx_operating_point = ctx.operating_point.get();
        ctx.operating_point_idc.set(
            operating_points[if ctx_operating_point < operating_points.len() {
                ctx_operating_point
            } else {
                0
            }]
            .idc,
        );

        // frame_width_bits_minus_1	f(4)
        let frame_width_bits = buf.get_bits(4) as u8 + 1;

        // frame_height_bits_minus_1	f(4)
        let frame_height_bits = buf.get_bits(4) as u8 + 1;

        // max_frame_width_minus_1	f(n)
        let max_frame_width = buf.get_bits(frame_width_bits as usize) as u16 + 1;

        // max_frame_height_minus_1	f(n)
        let max_frame_height = buf.get_bits(frame_height_bits as usize) as u16 + 1;

        let frame_id_numbers_present = if !reduced_still_picture_header {
            // frame_id_numbers_present_flag	f(1)
            if buf.get_bit() {
                Some(FrameIdNumbersPresent::decode(buf.as_mut()))
            } else {
                None
            }
        } else {
            None
        };

        // use_128x128_superblock	f(1)
        let use_128x128_superblock = buf.get_bit();

        // enable_filter_intra	f(1)
        let enable_filter_intra = buf.get_bit();

        // enable_intra_edge_filter	f(1)
        let enable_intra_edge_filter = buf.get_bit();

        let mut enable_interintra_compound = false;
        let mut enable_masked_compound = false;
        let mut enable_warped_motion = false;
        let mut enable_dual_filter = false;
        let mut enable_order_hint = false;
        let mut enable_jnt_comp = false;
        let mut enable_ref_frame_mvs = false;
        let mut seq_force_screen_content_tools = SELECT_SCREEN_CONTENT_TOOLS;
        let mut seq_force_integer_mv = SELECT_INTEGER_MV;

        if reduced_still_picture_header {
            ctx.order_hint_bits.set(0);
        } else {
            // enable_interintra_compound	f(1)
            enable_interintra_compound = buf.get_bit();

            // enable_masked_compound	f(1)
            enable_masked_compound = buf.get_bit();

            // enable_warped_motion	f(1)
            enable_warped_motion = buf.get_bit();

            // enable_dual_filter	f(1)
            enable_dual_filter = buf.get_bit();

            // enable_order_hint	f(1)
            enable_order_hint = buf.get_bit();
            if enable_order_hint {
                // enable_jnt_comp	f(1)
                enable_jnt_comp = buf.get_bit();

                // enable_ref_frame_mvs	f(1)
                enable_ref_frame_mvs = buf.get_bit();
            }

            // seq_choose_screen_content_tools	f(1)
            let seq_choose_screen_content_tools = buf.get_bit();
            if !seq_choose_screen_content_tools {
                // seq_force_screen_content_tools	f(1)
                seq_force_screen_content_tools = buf.get_bit() as u8;
            }

            if seq_force_screen_content_tools > 0 {
                // seq_choose_integer_mv	f(1)
                let seq_choose_integer_mv = buf.get_bit();
                if !seq_choose_integer_mv {
                    // seq_force_integer_mv	f(1)
                    seq_force_integer_mv = buf.get_bit() as u8;
                }
            }

            ctx.order_hint_bits.set(if enable_order_hint {
                // order_hint_bits_minus_1	f(3)
                buf.get_bits(3) as usize + 1
            } else {
                0
            });
        }

        // enable_superres	f(1)
        let enable_superres = buf.get_bit();

        // enable_cdef	f(1)
        let enable_cdef = buf.get_bit();

        // enable_restoration	f(1)
        let enable_restoration = buf.get_bit();

        let color_config = ColorConfig::decode(ctx, buf, seq_profile)?;

        // film_grain_params_present	f(1)
        let film_grain_params_present = buf.get_bit();

        Ok(Self {
            seq_profile,
            still_picture,
            reduced_still_picture_header,
            timing_info,
            decoder_model_info,
            initial_display_delay_present_flag,
            operating_points,
            frame_width_bits,
            frame_height_bits,
            max_frame_width,
            max_frame_height,
            frame_id_numbers_present,
            use_128x128_superblock,
            enable_filter_intra,
            enable_intra_edge_filter,
            enable_interintra_compound,
            enable_masked_compound,
            enable_warped_motion,
            enable_dual_filter,
            enable_order_hint,
            enable_jnt_comp,
            enable_ref_frame_mvs,
            seq_force_screen_content_tools,
            seq_force_integer_mv,
            enable_superres,
            enable_cdef,
            enable_restoration,
            color_config,
            film_grain_params_present,
        })
    }
}
