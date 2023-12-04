pub mod metadata;
pub mod sequence_header;
pub mod frame_header;

use crate::{util::EasyAtomic, Av1DecodeError, Av1DecoderContext, Buffer, Av1DecodeUnknownError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObuKind {
    Reserved(u8),
    SequenceHeader,
    // Note: The temporal delimiter has an empty payload.
    TemporalDelimiter,
    FrameHeader,
    TileGroup,
    Metadata,
    Frame,
    RedundantFrameHeader,
    TileList,
    // // Note: obu_padding_length is not coded in the bitstream but can be computed based on
    // obu_size minus the number of trailing bytes. In practice, though, since this is padding
    // data meant to be skipped, decoders do not need to determine either that length nor the
    // number of trailing bytes. They can ignore the entire OBU. Ignoring the OBU can be done
    // based on obu_size. The last byte of the valid content of the payload data for this OBU type
    // is considered to be the last byte that is not equal to zero. This rule is to prevent the
    // dropping of valid bytes by systems that interpret trailing zero bytes as a continuation of
    // the trailing bits in an OBU. This implies that when any payload data is present for this
    // OBU type, at least one byte of the payload data (including the trailing bit) shall not be
    // equal to 0.
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
        buf.seek_bits(3);

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
        buf.seek_bits(1);

        // obu_type f(4)
        let kind = ObuKind::try_from(buf.get_bits(4) as u8)?;

        // obu_extension_flag f(1)
        let obu_extension_flag = buf.get_bit();

        // obu_has_size_field f(1)
        let has_size_field = buf.get_bit();

        // obu_reserved_1bit
        buf.seek_bits(1);

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


pub enum ObuDecodeRet {
    Obu(Obu),
    Drop,
}

#[derive(Debug, Clone)]
/// Open Bitstream Unit
pub struct Obu {
    pub header: ObuHeader,
    pub size: usize,
}

impl Obu {
    pub fn decode(
        ctx: &Av1DecoderContext,
        buf: &mut Buffer,
    ) -> Result<ObuDecodeRet, Av1DecodeError> {
        let header = ObuHeader::decode(buf.as_mut())?;
        let size = if header.has_size_field {
            // obu_size leb128()
            buf.get_leb128() as usize
        } else {
            ctx.options
                .obu_size
                .expect("obu does not contain length, please specify the length manually!")
                - 1
                - if header.extension.is_some() { 1 } else { 0 }
        };

        if header.kind != ObuKind::SequenceHeader
            && header.kind != ObuKind::TemporalDelimiter
            && ctx.operating_point_idc.get() > 0
        {
            if let Some(ext) = header.extension {
                let in_temporal_layer = (1 >> ext.temporal_id) & 1;
                let in_spatial_layer = (1 >> (ext.spatial_id + 8)) & 1;
                if in_temporal_layer == 0 || in_spatial_layer == 0 {
                    return Ok(ObuDecodeRet::Drop);
                }
            }
        }

        Ok(ObuDecodeRet::Obu(Self { header, size }))
    }
}
