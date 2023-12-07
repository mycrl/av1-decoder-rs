use crate::{
    constants::{NUM_REF_FRAMES, PRIMARY_REF_NONE, SELECT_INTEGER_MV, SELECT_SCREEN_CONTENT_TOOLS},
    obu::sequence_header::FrameIdNumbersPresent,
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
pub struct UncompressedHeader {}

impl UncompressedHeader {
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

        if (!ctx.frame_is_intra || refresh_frame_flags != all_frames)
            && error_resilient_mode
            && sequence_header.enable_order_hint
        {}

        Ok(Self {})
    }
}
