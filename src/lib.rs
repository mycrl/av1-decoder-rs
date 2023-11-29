mod bitstream;
mod nal;

use std::collections::HashMap;

use bytes::BytesMut;
use nal::pps::Pps;
pub use nal::{
    sps::{Sps, SpsDecodeError, SpsDecodeErrorKind},
    Nalu, NaluDecodeError, Nalunit, Nri, Nut,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum H264DecodeError {
    NaluDecodeError(#[from] NaluDecodeError),
}

impl std::fmt::Display for H264DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Default)]
pub(crate) struct Session {
    pub spss: HashMap<usize, Sps>,
    pub ppss: HashMap<usize, Pps>,
}

pub struct H264Decoder {
    session: Session,
    bytes: BytesMut,
    index: usize,
}

impl H264Decoder {
    pub fn new() -> Self {
        Self {
            session: Session::default(),
            bytes: BytesMut::new(),
            index: 0,
        }
    }

    pub fn parse_next(&mut self, buf: &[u8]) -> Result<Vec<Nalu>, H264DecodeError> {
        self.bytes.extend_from_slice(buf);

        let mut nalus = vec![];
        if self.bytes.len() < 5 {
            return Ok(vec![]);
        }

        loop {
            if self.index + 1 >= self.bytes.len() {
                break;
            }

            if let Some(size) = Self::find_start_code(&self.bytes[self.index..]) {
                let buf = self.bytes.split_to(self.index + size);
                self.index = size;
                if buf.len() > size {
                    // nalus.push(Nalu::try_from(&buf[..])?);
                }
            } else {
                self.index += 1;
            }
        }

        Ok(nalus)
    }

    fn find_start_code(buf: &[u8]) -> Option<usize> {
        if buf.len() < 3 {
            return None;
        }

        // 0x000001
        if buf[0] == 0 && buf[1] == 0 && buf[2] == 1 {
            Some(3)
        } else {
            if buf.len() < 4 {
                return None;
            }

            // 0x00000001
            if buf[0] == 0 && buf[1] == 0 && buf[2] == 0 && buf[3] == 1 {
                Some(4)
            } else {
                None
            }
        }
    }
}
