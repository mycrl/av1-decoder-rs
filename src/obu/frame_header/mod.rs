mod uncompressed_header;

use crate::{util::EasyAtomic, Av1DecodeError, Av1DecoderContext, Buffer};

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
