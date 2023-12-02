mod buffer;
mod obu;
mod util;

use std::sync::atomic::AtomicBool;

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

pub struct Av1DecoderSession {
    pub options: Av1DecoderOptions,
    pub operating_point_idc: AtomicBool,
}
