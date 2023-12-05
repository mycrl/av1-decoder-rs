mod buffer;
mod constants;
mod obu;
mod util;

use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU8, AtomicUsize};

pub use buffer::Buffer;
use obu::sequence_header::SequenceHeader;
pub use obu::{Obu, ObuHeader, ObuHeaderExtension, ObuKind};
use util::AtomicOption;

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

pub struct Av1DecoderContext {
    pub options: Av1DecoderOptions,
    pub operating_point_idc: AtomicU16,
    pub operating_point: AtomicUsize,
    pub order_hint_bits: AtomicUsize,
    pub bit_depth: AtomicU8,
    pub num_planes: AtomicU8,
    pub seen_frame_header: AtomicBool,
    pub sequence_header: AtomicOption<SequenceHeader>,
    pub frame_is_intra: AtomicBool,
}
