use crate::{
    constants::NUM_REF_FRAMES, Av1DecodeError, Av1DecodeUnknownError, Av1DecoderContext, Buffer,
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
    pub fn decode(ctx: &mut Av1DecoderContext, buf: &mut Buffer) -> Result<Self, Av1DecodeError> {
        let sequence_header = ctx
            .sequence_header
            .as_ref()
            .expect("sequence header cannot be found, this is a undefined behavior!");

        let mut id_len = 0;
        if let Some(value) = &sequence_header.frame_id_numbers_present {
            id_len = value.additional_frame_id_length + value.delta_frame_id_length + 3;
        }

        let all_frames = (1 << NUM_REF_FRAMES) - 1;

        let mut show_existing_frame = false;
        let mut frame_type = FrameType::KeyFrame;
        let mut show_frame = true;
        let mut showable_frame = false;
        let mut refresh_frame_flags = 0;

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
                    let display_frame_id = buf.get_bits(id_len as usize);
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

            let error_resilient_mode = if frame_type == FrameType::SwitchFrame
                || frame_type == FrameType::KeyFrame && show_frame
            {
                true
            } else {
                // error_resilient_mode	f(1)
                buf.get_bit()
            };
        }

        if frame_type == FrameType::KeyFrame && show_frame {
            for i in 0..NUM_REF_FRAMES {}
        }

        Ok(Self {})
    }
}
