use crate::{
    bitstream::{BitRead, Bits, ExpGolomb},
    SpsDecodeError,
};

#[derive(Debug, Clone)]
pub struct Hrd {
    /// cpb_cnt_minus1 plus 1 specifies the number of alternative CPB
    /// specifications in the bitstream. The value of cpb_cnt_minus1 shall
    /// be in the range of 0 to 31, inclusive. When low_delay_hrd_flag is equal
    /// to 1, cpb_cnt_minus1 shall be equal to 0. When cpb_cnt_minus1 is not
    /// present, it shall be inferred to be equal to 0.
    pub cpb_cnt_minus1: usize,
    /// bit_rate_scale (together with bit_rate_value_minus1[ SchedSelIdx ])
    /// specifies the maximum input bit rate of the SchedSelIdx-th CPB.
    pub bit_rate_scale: u8,
    /// cpb_size_scale (together with cpb_size_value_minus1[ SchedSelIdx ])
    /// specifies the CPB size of the SchedSelIdx-th CPB.
    pub cpb_size_scale: u8,
    /// bit_rate_value_minus1[ SchedSelIdx ] (together with bit_rate_scale)
    /// specifies the maximum input bit rate for the SchedSelIdx-th CPB.
    pub bit_rate_value_minus1: Vec<usize>,
    /// cpb_size_value_minus1[ SchedSelIdx ] is used together with
    /// cpb_size_scale to specify the SchedSelIdx-th CPB size.
    /// cpb_size_value_minus1[ SchedSelIdx ] shall be in the range of 0 to 232 −
    /// 2, inclusive. For any SchedSelIdx greater than
    /// 0, cpb_size_value_minus1[ SchedSelIdx ] shall be less than or equal to
    /// cpb_size_value_minus1[ SchedSelIdx −1 ].
    pub cpb_size_value_minus1: Vec<usize>,
    /// cbr_flag[ SchedSelIdx ] equal to 0 specifies that to decode this
    /// bitstream by the HRD using the SchedSelIdx-th CPB specification, the
    /// hypothetical stream delivery scheduler (HSS) operates in an intermittent
    /// bit rate mode. cbr_flag[ SchedSelIdx ] equal to 1 specifies that the
    /// HSS operates in a constant bit rate (CBR) mode. When the
    /// cbr_flag[ SchedSelIdx ] syntax element is not present, the value of
    /// cbr_flag shall be inferred to be equal to 0.
    pub cbr_flags: Vec<bool>,
    /// initial_cpb_removal_delay_length_minus1 specifies the length in bits of
    /// the initial_cpb_removal_delay[ SchedSelIdx ] and
    /// initial_cpb_removal_delay_offset[ SchedSelIdx ] syntax elements of the
    /// buffering period SEI message. The length of initial_cpb_removal_delay[
    /// SchedSelIdx ] and of initial_cpb_removal_delay_offset[ SchedSelIdx ]
    /// is initial_cpb_removal_delay_length_minus1 + 1. When the
    /// initial_cpb_removal_delay_length_minus1 syntax element is present in
    /// more than one hrd_parameters( ) syntax structure within the VUI
    /// parameters syntax structure, the value of the
    /// initial_cpb_removal_delay_length_minus1 parameters shall be equal in
    /// both hrd_parameters( ) syntax structures. When the
    /// initial_cpb_removal_delay_length_minus1 syntax element is not present,
    /// it shall be inferred to be equal to 23.
    pub initial_cpb_removal_delay_length_minus1: u8,
    /// cpb_removal_delay_length_minus1 specifies the length in bits of the
    /// cpb_removal_delay syntax element. The length
    /// of the cpb_removal_delay syntax element of the picture timing SEI
    /// message is cpb_removal_delay_length_minus1 + 1.
    /// When the cpb_removal_delay_length_minus1 syntax element is present in
    /// more than one hrd_parameters( ) syntax 386 structure within the VUI
    /// parameters syntax structure, the value of the
    /// cpb_removal_delay_length_minus1 parameters shall be equal in both
    /// hrd_parameters( ) syntax structures. When the
    /// cpb_removal_delay_length_minus1 syntax element is not present, it
    /// shall be inferred to be equal to 23.
    pub cpb_removal_delay_length_minus1: u8,
    /// dpb_output_delay_length_minus1 specifies the length in bits of the
    /// dpb_output_delay syntax element. The length of the dpb_output_delay
    /// syntax element of the picture timing SEI message is
    /// dpb_output_delay_length_minus1 + 1. When
    /// the dpb_output_delay_length_minus1 syntax element is present in more
    /// than one hrd_parameters( ) syntax structure within the VUI
    /// parameters syntax structure, the value of the
    /// dpb_output_delay_length_minus1 parameters shall be equal in both
    /// hrd_parameters( ) syntax structures. When the
    /// dpb_output_delay_length_minus1 syntax element is not present, it
    /// shall be inferred to be equal to 23.
    pub dpb_output_delay_length_minus1: u8,
    /// time_offset_length greater than 0 specifies the length in bits of the
    /// time_offset syntax element. time_offset_length equal to 0 specifies
    /// that the time_offset syntax element is not present. When the
    /// time_offset_length syntax element is present in more than one
    /// hrd_parameters( ) syntax structure within the VUI parameters syntax
    /// structure, the value of the time_offset_length parameters shall be
    /// equal in both hrd_parameters( ) syntax structures. When the
    /// time_offset_length syntax element is not present, it shall be
    /// inferred to be equal to 24.
    pub time_offset_length: u8,
}

impl TryFrom<&mut Bits<'_>> for Hrd {
    type Error = SpsDecodeError;

    fn try_from(value: &mut Bits) -> Result<Self, Self::Error> {
        // cpb_cnt_minus1 0 | 5 ue(v)
        let cpb_cnt_minus1 = value.get_unsigned();

        // bit_rate_scale 0 | 5 u(4)
        let bit_rate_scale = value.get_bits(4) as u8;

        // cpb_size_scale 0 | 5 u(4)
        let cpb_size_scale = value.get_bits(4) as u8;

        let mut bit_rate_value_minus1 = Vec::with_capacity(cpb_cnt_minus1);
        let mut cpb_size_value_minus1 = Vec::with_capacity(cpb_cnt_minus1);
        let mut cbr_flags = Vec::with_capacity(cpb_cnt_minus1);
        for _ in 0..cpb_cnt_minus1 {
            // bit_rate_value_minus1[ SchedSelIdx ] 0 | 5 ue(v)
            bit_rate_value_minus1.push(value.get_unsigned());
            // cpb_size_value_minus1[ SchedSelIdx ] 0 | 5 ue(v)
            cpb_size_value_minus1.push(value.get_unsigned());
            // cbr_flag[ SchedSelIdx ] 0 | 5 u(1)
            cbr_flags.push(value.get_bit());
        }

        // initial_cpb_removal_delay_length_minus1 0 | 5 u(5)
        let initial_cpb_removal_delay_length_minus1 = value.get_bits(5) as u8;

        // cpb_removal_delay_length_minus1 0 | 5 u(5)
        let cpb_removal_delay_length_minus1 = value.get_bits(5) as u8;

        // dpb_output_delay_length_minus1 0 | 5 u(5)
        let dpb_output_delay_length_minus1 = value.get_bits(5) as u8;

        // time_offset_length 0 | 5 u(5)
        let time_offset_length = value.get_bits(5) as u8;

        Ok(Self {
            cpb_cnt_minus1,
            bit_rate_scale,
            cpb_size_scale,
            bit_rate_value_minus1,
            cpb_size_value_minus1,
            cbr_flags,
            initial_cpb_removal_delay_length_minus1,
            cpb_removal_delay_length_minus1,
            dpb_output_delay_length_minus1,
            time_offset_length,
        })
    }
}
