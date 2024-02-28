use std::marker::PhantomData;

use crate::{
    mode::{DualMode, Mode},
    PointInfo, PointInfos, SingleMode,
};

pub type Dual128OusterPacket = OusterPacket<16, 128, DualMode<16, 128>>;
pub type Single128OusterPacket = OusterPacket<16, 128, SingleMode<16, 128>>;
pub type Dual64OusterPacket = OusterPacket<16, 64, DualMode<16, 128>>;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct OusterPacket<const TCOLUMNS: usize, const TLAYERS: usize, TProfile: Mode> {
    pub header: OusterPacketHeader,
    pub columns: [Column<TLAYERS, TProfile>; TCOLUMNS],
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

impl<const TCOLUMNS: usize, const TLAYERS: usize, TProfile: Mode> Default
    for OusterPacket<TCOLUMNS, TLAYERS, TProfile>
{
    fn default() -> Self {
        Self {
            header: Default::default(),
            columns: [Default::default(); TCOLUMNS],
            reserved: [0; 8],
        }
    }
}
impl<const TCOLUMNS: usize, const TLAYERS: usize, TProfile: Mode>
    OusterPacket<TCOLUMNS, TLAYERS, TProfile>
{
    // Not yet aware of Endianness... The buffer needs to be modified in that case and data_accessors of irregular bitsizes have to be adapted too
    // mut allows to implement this in the future without breaking changes
    #[cfg(target_endian = "little")]
    pub unsafe fn from_aligned_memory(buffer: &[u8]) -> &Self {
        if (buffer.as_ptr()) as usize % 32 != 0 {
            panic!("Buffer has to be aligned");
        }

        unsafe { &*(buffer.as_ptr() as *const Self) }
    }

    pub fn from_maybe_unaligned(buffer: &[u8]) -> Result<Self, SizeMismatchError> {
        let mut inner = Self::default();
        let s = std::mem::size_of::<Self>();
        {
            let inner_ptr: *mut u8 = (&mut inner) as *mut Self as _;
            let as_buf = unsafe { std::slice::from_raw_parts_mut(inner_ptr, s) };
            if as_buf.len() != buffer.len() {
                return Err(SizeMismatchError {
                    expected: as_buf.len(),
                    actual: buffer.len(),
                });
            }
            as_buf.copy_from_slice(buffer);
        }
        Ok(inner)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Expected {expected}, got {actual}")]
pub struct SizeMismatchError {
    expected: usize,
    actual: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Column<const TLAYERS: usize, TProfile: Mode> {
    pub channels_header: ChannelsHeader,
    pub channels: [TProfile::Channel; TLAYERS],
    phantom: PhantomData<TProfile>,
}

impl<const TLAYERS: usize, TProfile: Mode> Default for Column<TLAYERS, TProfile> {
    fn default() -> Self {
        Self {
            channels_header: ChannelsHeader::default(),
            channels: [TProfile::Channel::default(); TLAYERS],
            phantom: PhantomData,
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
pub struct DualChannel {
    pub info_ret1: RangeData,
    pub info_ret2: RangeData,
    pub signal_ret_1: u16,
    pub signal_ret_2: u16,
    pub nir: u16,
    _reserved: u16,
}

impl PointInfos for DualChannel {
    fn get_primary_infos_uncorrected(&self) -> crate::PointInfo {
        PointInfo {
            distance: self.info_ret1.get_distance(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SingleChannel {
    pub range_and_reserved: u32,
    pub reflectifity: u8,
    _reserved: u8,
    pub signal: u16,
    pub nir: u16,
    _reserved2: u16,
}

impl PointInfos for SingleChannel {
    fn get_primary_infos_uncorrected(&self) -> crate::PointInfo {
        PointInfo {
            distance: self.range_and_reserved & ((1 << 20) - 1),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct LowDataChannel {
    pub data: u32,
}

impl PointInfos for LowDataChannel {
    fn get_primary_infos_uncorrected(&self) -> crate::PointInfo {
        PointInfo {
            distance: (self.data & ((1 << 17) - 1)) * 8,
        }
    }
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
        assert_eq!(128 / 8, std::mem::size_of::<super::DualChannel>());
        assert_eq!(96 / 8, std::mem::size_of::<super::SingleChannel>());
        assert_eq!(32 / 8, std::mem::size_of::<super::RangeData>());
        assert_eq!(33024, std::mem::size_of::<Dual128OusterPacket>());
        assert_eq!(24832, std::mem::size_of::<Single128OusterPacket>());
    }
}
