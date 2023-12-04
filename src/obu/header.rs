use crate::{buffer::Buffer, Av1DecodeError, Av1DecodeUnknownError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObuKind {
    Reserved(u8),
    SequenceHeader,
    TemporalDelimiter,
    FrameHeader,
    TileGroup,
    Metadata,
    Frame,
    RedundantFrameHeader,
    TileList,
    Padding,
}

impl TryFrom<u8> for ObuKind {
    type Error = Av1DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 | 9..=14 => Self::Reserved(value),
            1 => Self::SequenceHeader,
            2 => Self::TemporalDelimiter,
            3 => Self::FrameHeader,
            4 => Self::TileGroup,
            5 => Self::Metadata,
            6 => Self::Frame,
            7 => Self::RedundantFrameHeader,
            8 => Self::TileList,
            15 => Self::Padding,
            _ => {
                return Err(Av1DecodeError::Unknown(
                    Av1DecodeUnknownError::ObuHeaderKind,
                ))
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ObuHeaderExtension {
    pub temporal_id: u8,
    pub spatial_id: u8,
}

impl ObuHeaderExtension {
    pub fn decode(buf: &mut Buffer<'_>) -> Result<Self, Av1DecodeError> {
        // temporal_id f(3)
        let temporal_id = buf.get_bits(3) as u8;

        // spatial_id f(2)
        let spatial_id = buf.get_bits(2) as u8;

        // extension_header_reserved_3bits
        buf.seek(3);

        Ok(Self {
            temporal_id,
            spatial_id,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ObuHeader {
    pub kind: ObuKind,
    pub has_size_field: bool,
    pub extension: Option<ObuHeaderExtension>,
}

impl ObuHeader {
    pub fn decode(buf: &mut Buffer<'_>) -> Result<Self, Av1DecodeError> {
        // obu_forbidden_bit f(1)
        buf.seek(1);

        // obu_type f(4)
        let kind = ObuKind::try_from(buf.get_bits(4) as u8)?;

        // obu_extension_flag f(1)
        let obu_extension_flag = buf.get_bit();

        // obu_has_size_field f(1)
        let has_size_field = buf.get_bit();

        // obu_reserved_1bit
        buf.seek(1);

        let extension = if obu_extension_flag {
            Some(ObuHeaderExtension::decode(buf.as_mut())?)
        } else {
            None
        };

        Ok(Self {
            kind,
            has_size_field,
            extension,
        })
    }
}
