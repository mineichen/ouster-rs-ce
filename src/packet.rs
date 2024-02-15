pub type Dual128OusterPacket = OusterPacket<16, 128>;
pub type Dual64OusterPacket = OusterPacket<16, 64>;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct OusterPacket<const TCOLUMNS: usize, const TCHANNELS: usize> {
    pub header: OusterPacketHeader,
    pub columns: [Column<TCHANNELS>; TCOLUMNS],
    pub reserved: [u32; 8],
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct OusterPacketHeader {
    pub packet_type: u16,
    pub frame_id: u16,
    pub init_id_part1: u16,
    pub init_id_part2: u8,
    pub serial_no_1: u8,
    pub serial_no_2: u32,
    _reserved_1: u32,
    pub shutdown_countdown: u8,
    pub shot_limiting_countdown: u8,
    pub shutdown_status_and_reserve: u8,
    pub shot_limiting_status_and_reserve: u8,
    _reserved_2: [u32; 3],
}

impl<const TCOLUMNS: usize, const TCHANNELS: usize> Default for OusterPacket<TCOLUMNS, TCHANNELS> {
    fn default() -> Self {
        Self {
            header: Default::default(),
            columns: [Default::default(); TCOLUMNS],
            reserved: [0; 8],
        }
    }
}
impl<const TCOLUMNS: usize, const TCHANNELS: usize> OusterPacket<TCOLUMNS, TCHANNELS> {
    // Not yet aware of Endianness... The buffer needs to be modified in that case and data_accessors of irregular bitsizes have to be adapted too
    // mut allows to implement this in the future without breaking changes
    #[cfg(target_endian = "little")]
    pub fn from_aligned_memory(buffer: &[u8]) -> &Self {
        if (buffer.as_ptr()) as usize % 32 != 0 {
            panic!("Buffer has to be aligned");
        }

        unsafe { &*(buffer.as_ptr() as *const Self) }
    }

    pub fn from_maybe_unaligned(buffer: &[u8]) -> Self {
        let mut inner = Self::default();
        let s = std::mem::size_of::<Self>();
        {
            let inner_ptr: *mut u8 = (&mut inner) as *mut Self as _;
            let as_buf = unsafe { std::slice::from_raw_parts_mut(inner_ptr, s) };
            as_buf.copy_from_slice(buffer);
        }
        inner
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Column<const TCHANNELS: usize> {
    pub channels_header: ChannelsHeader,
    pub channels: [Channel; TCHANNELS],
}

impl<const TCHANNELS: usize> Default for Column<TCHANNELS> {
    fn default() -> Self {
        Self {
            channels_header: ChannelsHeader::default(),
            channels: [Channel::default(); TCHANNELS],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ChannelsHeader {
    // Single u64 would force ChannelsHeader to be 64bit aligned
    pub timestamp_a: u32,
    pub timestamp_b: u32,
    pub measurement_id: u16,
    pub status_and_reserve: u16,
}

impl ChannelsHeader {
    pub fn timestamp() -> u64 {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Channel {
    pub info_ret1: RangeData,
    pub info_ret2: RangeData,
    pub signal_ret_1: u16,
    pub signal_ret_2: u16,
    pub nir: u16,
    _reserved: u16,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct RangeData {
    raw: u32,
}

impl RangeData {
    pub fn get_distance(&self) -> u32 {
        self.raw & ((1 << 20) - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_correct_structsize() {
        assert_eq!(256 / 8, std::mem::size_of::<OusterPacketHeader>());
        assert_eq!(96 / 8, std::mem::size_of::<ChannelsHeader>());
        assert_eq!(128 / 8, std::mem::size_of::<super::Channel>());
        assert_eq!(32 / 8, std::mem::size_of::<super::RangeData>());
        assert_eq!(33024, std::mem::size_of::<Dual128OusterPacket>());
    }
}
