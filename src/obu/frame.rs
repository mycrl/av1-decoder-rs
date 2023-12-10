use super::frame_header::FrameHeader;

#[derive(Debug, Clone)]
pub struct Frame {
    pub frame_header: FrameHeader,
}
