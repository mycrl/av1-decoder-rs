mod buffer;
mod constants;
mod obu;
mod util;

pub use buffer::Buffer;
use constants::NUM_REF_FRAMES;
use obu::{frame_header::FrameHeader, sequence_header::SequenceHeader};
pub use obu::{Obu, ObuHeader, ObuHeaderExtension, ObuKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Av1DecodeUnknownError {
    ObuHeaderKind,
    Profile,
    ColorPrimaries,
    TransferCharacteristics,
    MatrixCoefficients,
    ChromaSamplePosition,
    MetadataType,
    ScalabilityModeIdc,
    FrameType,
    InterpolationFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Av1DecodeError {
    Unknown(Av1DecodeUnknownError),
}

impl std::error::Error for Av1DecodeError {}

impl std::fmt::Display for Av1DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Av1DecoderOptions {
    pub obu_size: Option<usize>,
}

pub struct Av1DecoderContextRef {
    pub sequence_header: SequenceHeader,
    pub frame_header: FrameHeader,
}

pub struct Av1DecoderContext {
    pub options: Av1DecoderOptions,
    pub operating_point_idc: u16,
    pub operating_point: usize,
    pub order_hint_bits: usize,
    pub bit_depth: u8,
    pub num_planes: u8,
    pub seen_frame_header: bool,
    pub sequence_header: Option<SequenceHeader>,
    pub frame_is_intra: bool,
    pub refs: [Option<Av1DecoderContextRef>; NUM_REF_FRAMES as usize],
    pub order_hint: u32,
    pub obu_header_extension: Option<ObuHeaderExtension>,
    pub frame_width: u16,
    pub frame_height: u16,
    pub superres_denom: u8,
    pub upscaled_width: u16,
    pub mi_cols: u32,
    pub mi_rows: u32,
    pub render_width: u16,
    pub render_height: u16,
    pub delta_frame_id: u32,
}
