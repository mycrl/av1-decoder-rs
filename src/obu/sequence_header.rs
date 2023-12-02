use crate::{Av1DcodeError, Av1DecoderSession, Buffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceProfile {
    Main,
    High,
    Professional,
}

impl TryFrom<u8> for SequenceProfile {
    type Error = Av1DcodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Main,
            1 => Self::High,
            2 => Self::Professional,
            _ => return Err(Av1DcodeError::InvalidProfile),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EqualPictureInterval {
    pub num_ticks_per_picture_minus_1: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct TimingInfo {
    pub num_units_in_display_tick: u32,
    pub time_scale: u32,
    pub equal_picture_interval: Option<EqualPictureInterval>,
}

impl TimingInfo {
    pub fn decode(buf: &mut Buffer<'_>) -> Self {
        // num_units_in_display_tick f(32)
        let num_units_in_display_tick = buf.get_bits(32);

        // time_scale f(32)
        let time_scale = buf.get_bits(32);

        // equal_picture_interval f(1)
        let equal_picture_interval = if buf.get_bit() {
            Some(EqualPictureInterval {
                // num_ticks_per_picture_minus_1 uvlc()
                num_ticks_per_picture_minus_1: buf.get_uvlc(),
            })
        } else {
            None
        };

        Self {
            num_units_in_display_tick,
            time_scale,
            equal_picture_interval,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DecoderModelInfo {
    pub buffer_delay_length_minus_1: u8,
    pub num_units_in_decoding_tick: u32,
    pub buffer_removal_time_length_minus_1: u8,
    pub frame_presentation_time_length_minus_1: u8,
}

impl DecoderModelInfo {
    pub fn decode(buf: &mut Buffer<'_>) -> Self {
        Self {
            // buffer_delay_length_minus_1 f(5)
            buffer_delay_length_minus_1: buf.get_bits(5) as u8,
            // num_units_in_decoding_tick f(32)
            num_units_in_decoding_tick: buf.get_bits(32),
            // buffer_removal_time_length_minus_1 f(5)
            buffer_removal_time_length_minus_1: buf.get_bits(5) as u8,
            // frame_presentation_time_length_minus_1 f(5)
            frame_presentation_time_length_minus_1: buf.get_bits(5) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OperatingParameters {
    pub decoder_buffer_delay: u32,
    pub encoder_buffer_delay: u32,
    pub low_delay_mode_flag: bool,
}

impl OperatingParameters {
    pub fn decode(buf: &mut Buffer<'_>, decoder_model_info: &DecoderModelInfo) -> Self {
        let size = (decoder_model_info.buffer_delay_length_minus_1 + 1) as usize;
        Self {
            // decoder_buffer_delay[ op ]	f(n)
            decoder_buffer_delay: buf.get_bits(size),
            // encoder_buffer_delay[ op ]	f(n)
            encoder_buffer_delay: buf.get_bits(size),
            // low_delay_mode_flag[ op ]	f(1)
            low_delay_mode_flag: buf.get_bit(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SequenceHeaderObu {
    pub seq_profile: SequenceProfile,
    pub still_picture: bool,
    pub reduced_still_picture_header: bool,
    pub timing_info: Option<TimingInfo>,
    pub decoder_model_info: Option<DecoderModelInfo>,
    pub initial_display_delay_present_flag: bool,
    pub operating_points_cnt_minus_1: u8,
    pub operating_point_idc: Vec<u16>,
    pub seq_level_idx: Vec<u8>,
    pub seq_tier: Vec<bool>,
    pub decoder_model_present_for_this_op: Vec<bool>,
    pub operating_parameters_infos: Vec<OperatingParameters>,
    pub initial_display_delay_present_for_this_op: Vec<bool>,
}

impl SequenceHeaderObu {
    pub fn decode(session: &Av1DecoderSession, buf: &mut Buffer) -> Result<Self, Av1DcodeError> {
        // seq_profile f(3)
        let seq_profile = SequenceProfile::try_from(buf.get_bits(3) as u8)?;

        // still_picture f(1)
        let still_picture = buf.get_bit();

        // reduced_still_picture_header f(1)
        let reduced_still_picture_header = buf.get_bit();

        let mut timing_info_present_flag = false;
        let mut timing_info = None;

        let mut decoder_model_info_present_flag = false;
        let mut decoder_model_info = None;

        let mut initial_display_delay_present_flag = false;
        let mut operating_points_cnt_minus_1 = 0;
        let mut operating_point_idc = Vec::with_capacity(10);
        let mut seq_level_idx = Vec::with_capacity(10);
        let mut seq_tier = Vec::with_capacity(10);

        let mut decoder_model_present_for_this_op = Vec::with_capacity(10);
        let mut operating_parameters_infos = Vec::with_capacity(10);

        let mut initial_display_delay_present_for_this_op = Vec::with_capacity(10);
        let mut initial_display_delay_minus_1 = Vec::with_capacity(10);

        if reduced_still_picture_header {
            operating_point_idc.push(0);

            // seq_level_idx[ 0 ] f(5)
            seq_level_idx[0] = buf.get_bits(5) as u8;
        } else {
            // timing_info_present_flag f(1)
            timing_info_present_flag = buf.get_bit();
            if timing_info_present_flag {
                timing_info = Some(TimingInfo::decode(buf.as_mut()));

                // decoder_model_info_present_flag f(1)
                decoder_model_info_present_flag = buf.get_bit();
                if decoder_model_info_present_flag {
                    decoder_model_info = Some(DecoderModelInfo::decode(buf.as_mut()));
                }
            }

            // initial_display_delay_present_flag	f(1)
            initial_display_delay_present_flag = buf.get_bit();

            // operating_points_cnt_minus_1	f(5)
            operating_points_cnt_minus_1 = buf.get_bits(5) as u8;

            for i in 0..operating_points_cnt_minus_1 as usize {
                // operating_point_idc[ i ]	f(12)
                operating_point_idc.push(buf.get_bits(12) as u16);

                // seq_level_idx[ i ]	f(5)
                let v = buf.get_bits(5) as u8;
                seq_level_idx.push(v);
                seq_tier.push(if v > 7 {
                    // seq_tier[ i ]	f(1)
                    buf.get_bit()
                } else {
                    false
                });

                if decoder_model_info_present_flag {
                    // decoder_model_present_for_this_op[ i ]	f(1)
                    let v = buf.get_bit();
                    decoder_model_present_for_this_op.push(v);
                    if v {
                        operating_parameters_infos.push(OperatingParameters::decode(
                            buf.as_mut(),
                            &decoder_model_info.unwrap(),
                        ));
                    }
                } else {
                    decoder_model_present_for_this_op.push(false);
                }

                if initial_display_delay_present_flag {
                    // initial_display_delay_present_for_this_op[ i ]	f(1)
                    let v = buf.get_bit();
                    initial_display_delay_present_for_this_op.push(v);
                    if v {
                        // initial_display_delay_minus_1[ i ]	f(4)
                        initial_display_delay_minus_1.push(buf.get_bits(4) as u8);
                    }
                }
            }
        }

        Ok(Self {
            seq_profile,
            still_picture,
            reduced_still_picture_header,
            timing_info,
            decoder_model_info,
            initial_display_delay_present_flag,
            operating_points_cnt_minus_1,
            operating_point_idc,
            seq_level_idx,
        })
    }
}
