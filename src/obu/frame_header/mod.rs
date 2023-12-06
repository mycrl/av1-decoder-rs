mod uncompressed_header;

use crate::{Av1DecodeError, Av1DecoderContext, Buffer};

#[derive(Debug, Clone)]
pub struct FrameHeader {}

impl FrameHeader {
    pub fn decode(ctx: &mut Av1DecoderContext, buf: &mut Buffer) -> Result<Self, Av1DecodeError> {
        if ctx.seen_frame_header {
            // frame_header_copy
        } else {
            ctx.seen_frame_header = true;
        }

        Ok(Self {})
    }
}
