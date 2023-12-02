mod buffer;
mod obu;
mod util;
mod constants;

use std::sync::atomic::{AtomicU16, AtomicUsize};

pub use buffer::Buffer;
pub use obu::{Obu, ObuHeader, ObuHeaderExtension, ObuKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Av1DcodeError {
    InvalidObuHeaderKind,
    InvalidProfile,
}

impl std::error::Error for Av1DcodeError {}

impl std::fmt::Display for Av1DcodeError {
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
}
