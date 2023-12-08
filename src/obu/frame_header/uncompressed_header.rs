use crate::{
    constants::{
        NUM_REF_FRAMES, PRIMARY_REF_NONE, SELECT_INTEGER_MV, SELECT_SCREEN_CONTENT_TOOLS,
        SUPERRES_DENOM_BITS, SUPERRES_DENOM_MIN, SUPERRES_NUM, REFS_PER_FRAME,
    },
    obu::sequence_header::{FrameIdNumbersPresent, SequenceHeader},
    Av1DecodeError, Av1DecodeUnknownError, Av1DecoderContext, Buffer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    KeyFrame,
    InterFrame,
    InterOnlyFrame,
    SwitchFrame,
}

impl TryFrom<u8> for FrameType {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::KeyFrame,
            1 => Self::InterFrame,
            2 => Self::InterOnlyFrame,
            3 => Self::SwitchFrame,
            _ => return Err(Av1DecodeError::Unknown(Av1DecodeUnknownError::FrameType)),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TemporalPointInfo {
    pub frame_presentation_time: u32,
}

impl TemporalPointInfo {
    pub fn decode(buf: &mut Buffer, frame_presentation_time_length: usize) -> Self {
        // frame_presentation_time	f(n)
        Self {
            frame_presentation_time: buf.get_bits(frame_presentation_time_length),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameSize {
    pub frame_width: Option<u16>,
    pub frame_height: Option<u16>,
    pub superres_params: SuperresParams,
}

impl FrameSize {
    fn compute_image_size(ctx: &mut Av1DecoderContext) {
        ctx.mi_cols = 2 * ((ctx.frame_width + 7) >> 3) as u32;
        ctx.mi_rows = 2 * ((ctx.frame_height + 7) >> 3) as u32;
    }

    pub fn decode(
        ctx: &mut Av1DecoderContext,
        frame_size_override: bool,
        seq_hdr: &SequenceHeader,
        buf: &mut Buffer,
    ) -> Self {
        let mut frame_width = None;
        let mut frame_height = None;
        if frame_size_override {
            // frame_width_minus_1	f(n)
            let width = buf.get_bits(seq_hdr.frame_width_bits as usize) as u16 + 1;

            // frame_height_minus_1	f(n)
            let height = buf.get_bits(seq_hdr.frame_height_bits as usize) as u16 + 1;

            frame_width = Some(width);
            frame_height = Some(height);
            ctx.frame_width = width;
            ctx.frame_height = height;
        } else {
            ctx.frame_width = seq_hdr.max_frame_width;
            ctx.frame_height = seq_hdr.max_frame_height;
        }

        let superres_params = SuperresParams::decode(ctx, seq_hdr, buf);
        Self::compute_image_size(ctx);
        Self {
            frame_width,
            frame_height,
            superres_params,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SuperresParams {
    pub use_superres: bool,
    pub coded_denom: Option<u8>,
}

impl SuperresParams {
    pub fn decode(ctx: &mut Av1DecoderContext, seq_hdr: &SequenceHeader, buf: &mut Buffer) -> Self {
        let use_superres = if seq_hdr.enable_superres {
            // use_superres	f(1)
            buf.get_bit()
        } else {
            false
        };

        let mut coded_denom = None;
        if use_superres {
            // coded_denom	f(SUPERRES_DENOM_BITS)
            let n = buf.get_bits(SUPERRES_DENOM_BITS as usize) as u8;
            ctx.superres_denom = n + SUPERRES_DENOM_MIN;
            coded_denom = Some(n);
        } else {
            ctx.superres_denom = SUPERRES_NUM;
        };

        ctx.upscaled_width = ctx.frame_width;
        ctx.frame_width = (ctx.upscaled_width * SUPERRES_NUM as u16
            + (ctx.superres_denom as u16 / 2))
            / ctx.superres_denom as u16;

        Self {
            use_superres,
            coded_denom,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderSize {
    pub render_and_frame_size_different: bool,
    pub render_width: Option<u16>,
    pub render_height: Option<u16>,
}

impl RenderSize {
    pub fn decode(ctx: &mut Av1DecoderContext, buf: &mut Buffer) -> Self {
        // render_and_frame_size_different	f(1)
        let render_and_frame_size_different = buf.get_bit();

        let mut render_width = None;
        let mut render_height = None;
        if render_and_frame_size_different {
            // render_width_minus_1	f(16)
            let width = buf.get_bits(16) as u16 + 1;

            // render_height_minus_1	f(16)
            let height = buf.get_bits(16) as u16 + 1;

            render_width = Some(width);
            render_height = Some(height);
            ctx.render_width = width;
            ctx.render_height = height;
        } else {
            ctx.render_width = ctx.upscaled_width;
            ctx.render_height = ctx.frame_height;
        }

        Self {
            render_and_frame_size_different,
            render_width,
            render_height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameSizeWithRefs {
    pub found_refs: Vec<bool>,
}

impl FrameSizeWithRefs {
    pub fn decode(ctx: &mut Av1DecoderContext, buf: &mut Buffer) {
        let mut found_refs = Vec::with_capacity(REFS_PER_FRAME as usize);
        for _ in 0..REFS_PER_FRAME {
            // found_ref	f(1)
            found_refs.push(buf.get_bit());
            if found_refs[found_refs.len() - 1] {
                // UpscaledWidth = RefUpscaledWidth[ ref_frame_idx[ i ] ]	 
                // FrameWidth = UpscaledWidth	 
                // FrameHeight = RefFrameHeight[ ref_frame_idx[ i ] ]	 
                // RenderWidth = RefRenderWidth[ ref_frame_idx[ i ] ]	 
                // RenderHeight = RefRenderHeight[ ref_frame_idx[ i ] ]	 
                // break           
            }
        }

        // if !found_refs {

        // }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationFilter {
    Eighttap,
    EighttapSmooth,
    EighttapSharp,
    Bilinear,
    Switchable,
}

impl TryFrom<u8> for InterpolationFilter {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Eighttap,
            1 => Self::EighttapSmooth,
            2 => Self::EighttapSharp,
            3 => Self::Bilinear,
            4 => Self::Switchable,
            _ => return Err(Av1DecodeError::Unknown(Av1DecodeUnknownError::InterpolationFilter)),
        })
    }
}

#[derive(Debug, Clone)]
pub struct UncompressedHeader {}

impl UncompressedHeader {
    // TODO
    fn mark_ref_frames(f: &FrameIdNumbersPresent, id_len: usize, current_frame_id: u32) {
        let diff_len = f.delta_frame_id_length;
        for i in 0..NUM_REF_FRAMES as usize {
            if current_frame_id > (1 << diff_len) {
            } else {
            }
        }
    }

    pub fn decode(ctx: &mut Av1DecoderContext, buf: &mut Buffer) -> Result<Self, Av1DecodeError> {
        let sequence_header = ctx
            .sequence_header
            .as_ref()
            .expect("sequence header cannot be found, this is a undefined behavior!");

        let mut id_len = 0;
        if let Some(value) = &sequence_header.frame_id_numbers_present {
            id_len = value.additional_frame_id_length as usize
                + value.delta_frame_id_length as usize
                + 3;
        }

        let all_frames = (1 << NUM_REF_FRAMES) - 1;

        let mut show_existing_frame = false;
        let mut frame_type = FrameType::KeyFrame;
        let mut show_frame = true;
        let mut showable_frame = false;
        let mut refresh_frame_flags = 0;
        let mut error_resilient_mode = false;

        if sequence_header.reduced_still_picture_header {
            ctx.frame_is_intra = true;
        } else {
            // show_existing_frame	f(1)
            show_existing_frame = buf.get_bit();

            if show_existing_frame {
                // frame_to_show_map_idx	f(3)
                let frame_to_show_map_idx = buf.get_bits(3) as u8;
                if let Some(decoder_model_info) = &sequence_header.decoder_model_info {
                    if !sequence_header
                        .timing_info
                        .map(|v| v.equal_picture_interval.is_some())
                        .unwrap_or(false)
                    {
                        let temporal_point_info = TemporalPointInfo::decode(
                            buf.as_mut(),
                            decoder_model_info.frame_presentation_time_length as usize,
                        );
                    }
                }

                if sequence_header.frame_id_numbers_present.is_some() {
                    // display_frame_id	f(idLen)
                    let display_frame_id = buf.get_bits(id_len);
                }

                // TODO
                // frame_type = RefFrameType[ frame_to_show_map_idx ]

                if frame_type == FrameType::KeyFrame {
                    refresh_frame_flags = all_frames;
                }

                if sequence_header.film_grain_params_present {
                    // TODO
                    // load_grain_params( frame_to_show_map_idx )
                }

                // TODO
                // return
            }

            // frame_type	f(2)
            frame_type = FrameType::try_from(buf.get_bits(2) as u8)?;
            ctx.frame_is_intra =
                frame_type == FrameType::InterOnlyFrame || frame_type == FrameType::KeyFrame;

            // show_frame	f(1)
            show_frame = buf.get_bit();

            if show_frame {
                if let Some(decoder_model_info) = &sequence_header.decoder_model_info {
                    if !sequence_header
                        .timing_info
                        .map(|v| v.equal_picture_interval.is_some())
                        .unwrap_or(false)
                    {
                        let temporal_point_info = TemporalPointInfo::decode(
                            buf.as_mut(),
                            decoder_model_info.frame_presentation_time_length as usize,
                        );
                    }
                }

                showable_frame = frame_type != FrameType::KeyFrame;
            } else {
                // showable_frame	f(1)
                showable_frame = buf.get_bit();
            }

            error_resilient_mode = if frame_type == FrameType::SwitchFrame
                || frame_type == FrameType::KeyFrame && show_frame
            {
                true
            } else {
                // error_resilient_mode	f(1)
                buf.get_bit()
            };
        }

        // TODO
        // if ( frame_type == KEY_FRAME && show_frame ) {
        //     for ( i = 0; i < NUM_REF_FRAMES; i++ ) {
        //         RefValid[ i ] = 0
        //         RefOrderHint[ i ] = 0
        //     }
        //     for ( i = 0; i < REFS_PER_FRAME; i++ ) {
        //         OrderHints[ LAST_FRAME + i ] = 0
        //     }
        // }

        // disable_cdf_update	f(1)
        let disable_cdf_update = buf.get_bit();
        let allow_screen_content_tools =
            if sequence_header.seq_force_screen_content_tools == SELECT_SCREEN_CONTENT_TOOLS {
                // allow_screen_content_tools	f(1)
                buf.get_bit()
            } else {
                sequence_header.seq_force_screen_content_tools != 0
            };

        let mut force_integer_mv = if allow_screen_content_tools {
            if sequence_header.seq_force_integer_mv == SELECT_INTEGER_MV {
                // force_integer_mv	f(1)
                buf.get_bit()
            } else {
                sequence_header.seq_force_integer_mv != 0
            }
        } else {
            false
        };

        if ctx.frame_is_intra {
            force_integer_mv = true;
        }

        let current_frame_id = if let Some(f) = &sequence_header.frame_id_numbers_present {
            // current_frame_id	f(idLen)
            buf.get_bits(id_len)

            // TODO
            // mark_ref_frames( idLen )
        } else {
            0
        };

        let frame_size_override = if frame_type == FrameType::SwitchFrame {
            true
        } else if sequence_header.reduced_still_picture_header {
            false
        } else {
            // frame_size_override_flag	f(1)
            buf.get_bit()
        };

        // order_hint	f(OrderHintBits)
        let order_hint = buf.get_bits(ctx.order_hint_bits);
        ctx.order_hint = order_hint;

        let primary_ref_frame = if ctx.frame_is_intra || error_resilient_mode {
            PRIMARY_REF_NONE
        } else {
            // primary_ref_frame	f(3)
            buf.get_bits(3) as u8
        };

        let mut buffer_removal_times = Vec::with_capacity(sequence_header.operating_points.len());
        if let Some(decoder_model_info) = sequence_header.decoder_model_info {
            // buffer_removal_time_present_flag	f(1)
            let buffer_removal_time_present_flag = buf.get_bit();
            if buffer_removal_time_present_flag {
                for operating_point in &sequence_header.operating_points {
                    if operating_point.operating_parameters_info.is_some() {
                        if let Some(extension) = ctx.obu_header_extension {
                            let op_pt_dic = operating_point.idc;
                            let in_temporal_layer = ((op_pt_dic >> extension.temporal_id) & 1) != 0;
                            let in_spatial_layer =
                                ((op_pt_dic >> (extension.spatial_id + 8)) & 1) != 0;
                            if op_pt_dic == 0 || (in_temporal_layer && in_spatial_layer) {
                                // buffer_removal_time[ opNum ]	f(n)
                                buffer_removal_times.push(buf.get_bits(
                                    decoder_model_info.buffer_removal_time_length as usize,
                                ));
                            }
                        }
                    }
                }
            }
        }

        let mut allow_high_precision_mv = false;
        let mut use_ref_frame_mvs = false;
        let mut allow_intrabc = false;

        refresh_frame_flags = if frame_type == FrameType::SwitchFrame
            || frame_type == FrameType::KeyFrame && show_frame
        {
            all_frames
        } else {
            buf.get_bits(8)
        };

        let mut ref_order_hints = None;
        if (!ctx.frame_is_intra || refresh_frame_flags != all_frames)
            && error_resilient_mode
            && sequence_header.enable_order_hint
        {
            let mut hints = [0u32; NUM_REF_FRAMES as usize];
            for i in 0..NUM_REF_FRAMES as usize {
                // ref_order_hint[ i ]	f(OrderHintBits)
                hints[i] = buf.get_bits(ctx.order_hint_bits);
            }

            ref_order_hints = Some(hints);
        }

        let mut allow_intrabc = false;
        if ctx.frame_is_intra {
            let frame_size = FrameSize::decode(unsafe {
                &mut *(ctx as *const Av1DecoderContext as *mut Av1DecoderContext)
            }, frame_size_override, sequence_header, buf);
            let render_size = RenderSize::decode(ctx, buf);
            if allow_screen_content_tools && ctx.upscaled_width == ctx.frame_width {
                // allow_intrabc	f(1)
                allow_intrabc = buf.get_bit();
            }
        } else {
            let mut frame_refs_short_signaling = false;
            if !sequence_header.enable_order_hint {
                // frame_refs_short_signaling	f(1)
                frame_refs_short_signaling = buf.get_bit();
                if frame_refs_short_signaling {
                    // last_frame_idx	f(3)
                    let last_frame_idx = buf.get_bits(3);

                    // gold_frame_idx	f(3)
                    let gold_frame_idx = buf.get_bits(3);

                    // TODO
                    // set_frame_refs()
                }
            }

            for _ in 0..REFS_PER_FRAME {
                if frame_refs_short_signaling {
                    // ref_frame_idx[ i ]	f(3)
                    let ref_frame_idx = buf.get_bits(3);
                }

                if let Some(frame_id_numbers_present) = &sequence_header.frame_id_numbers_present {
                    let n = frame_id_numbers_present.delta_frame_id_length;
                    // delta_frame_id_minus_1	f(n)
                    let delta_frame_id = buf.get_bits(n as usize) + 1;
                    ctx.delta_frame_id = delta_frame_id;

                    // expectedFrameId[ i ] = ((current_frame_id + (1 << idLen) -	 
                    // DeltaFrameId ) % (1 << idLen))
                }
            }

            if frame_size_override && !error_resilient_mode {
                
            }
        }

        Ok(Self {})
    }
}
