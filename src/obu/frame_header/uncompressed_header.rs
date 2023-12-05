use crate::{
    constants::NUM_REF_FRAMES, util::EasyAtomic, Av1DecodeError, Av1DecodeUnknownError,
    Av1DecoderContext, Buffer,
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
    pub fn decode(ctx: &Av1DecoderContext, buf: &mut Buffer) {
        let sequence_header = ctx
            .sequence_header
            .get()
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
        let mut refresh_frame = false;

        if sequence_header.reduced_still_picture_header {
            ctx.frame_is_intra.set(true);
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

                refresh_frame = false;
                if sequence_header.frame_id_numbers_present.is_some() {
                    // display_frame_id	f(idLen)
                    let display_frame = buf.get_bits(id_len as usize);
                }
            }
        }
    }
}
