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
pub struct UncompressedHeader {}

impl UncompressedHeader {
    pub fn decode(ctx: &Av1DecoderContext, buf: &mut Buffer) {
        let sequence_header = ctx
            .sequence_header
            .get()
            .expect("sequence header cannot be found, this is a undefined behavior!");

        if let Some(value) = &sequence_header.frame_id_numbers_present {
            let id_len = value.additional_frame_id_length + value.delta_frame_id_length + 3;
        }

        let all_frames = (1 << NUM_REF_FRAMES) - 1;

        let mut show_existing_frame = false;
        let mut frame_type = FrameType::KeyFrame;
        let mut show_frame = true;
        let mut showable_frame = false;

        if sequence_header.reduced_still_picture_header {
            ctx.frame_is_intra.set(true);
        } else {
            // show_existing_frame	f(1)
            show_existing_frame = buf.get_bit();

            if show_existing_frame {
                
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameHeader {}

impl FrameHeader {
    pub fn decode(ctx: &Av1DecoderContext, buf: &mut Buffer) -> Result<Self, Av1DecodeError> {
        if ctx.seen_frame_header.get() {
            // frame_header_copy
        } else {
            ctx.seen_frame_header.set(true);
        }

        Ok(Self {})
    }
}
