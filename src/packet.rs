use std::{marker::PhantomData, time::Duration};

use bytemuck::Zeroable;

use crate::{
    profile::{DualProfile, Profile},
    PointChannelInfo, PointInfo, PointInfos, PrimaryPointInfo, SingleProfile,
};

pub type Dual128OusterPacket = OusterPacket<DualProfile<16, 128>>;
pub type Single128OusterPacket = OusterPacket<SingleProfile<16, 128>>;
pub type Dual64OusterPacket = OusterPacket<DualProfile<16, 64>>;

#[repr(C)]
#[derive(Debug, Clone, Zeroable)]
pub struct OusterPacket<TProfile: Profile> {
    pub header: OusterPacketHeader,
    pub columns: TProfile::Columns,
    pub reserved: [u32; 8],
}

#[repr(C)]
#[derive(Debug, Default, Clone, Zeroable)]
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

impl<TProfile: Profile> Default for OusterPacket<TProfile> {
    fn default() -> Self {
        Self {
            header: Default::default(),
            columns: TProfile::initialize_columns(),
            reserved: [0; 8],
        }
    }
}
impl<TProfile: Profile> OusterPacket<TProfile> {
    /// Not yet aware of Endianness... The buffer needs to be modified in that case and data_accessors of irregular bitsizes have to be adapted too
    /// mut allows to implement this in the future without breaking changes
    /// # Safety
    /// Memory has to be aligned with OusterPacket<TProfile>
    #[cfg(target_endian = "little")]
    pub unsafe fn from_aligned_memory(buffer: &[u8]) -> &Self {
        if (buffer.as_ptr()) as usize % 32 != 0 {
            panic!("Buffer has to be aligned");
        }

        unsafe { &*(buffer.as_ptr() as *const Self) }
    }

    pub fn as_slice(&self) -> &[u8] {
        let this: *const u8 = std::ptr::from_ref(self) as _;
        unsafe { std::slice::from_raw_parts(this, std::mem::size_of::<Self>()) }
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
    pub expected: usize,
    pub actual: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable)]
pub struct Column<TProfile: Profile> {
    pub channels_header: ChannelsHeader,
    pub channels: TProfile::Channels,
    phantom: PhantomData<TProfile>,
}

impl<TProfile: Profile> Default for Column<TProfile> {
    fn default() -> Self {
        Self {
            channels_header: ChannelsHeader::default(),
            channels: TProfile::initialize_channels(),
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable)]
pub struct ChannelsHeader {
    // Single u64 would force ChannelsHeader to be 64bit aligned
    timestamp_a: u32,
    timestamp_b: u32,
    pub measurement_id: u16,
    pub status_and_reserve: u16,
}

impl ChannelsHeader {
    pub fn timestamp(&self) -> Duration {
        let mut bytes = [0; 8];

        bytes[0..4].copy_from_slice(&self.timestamp_a.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.timestamp_b.to_le_bytes());
        Duration::from_nanos(u64::from_le_bytes(bytes))
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable)]
pub struct DualChannel {
    pub info_ret1: RangeData,
    pub info_ret2: RangeData,
    pub signal_ret_1: u16,
    pub signal_ret_2: u16,
    pub nir: u16,
    _reserved: u16,
}

impl PointInfos for DualChannel {
    type Signal = u16;
    type Infos = [PointChannelInfo<Self::Signal>; 2];
    fn get_primary_infos(&self, n_vec: u32) -> crate::PrimaryPointInfo<Self::Signal> {
        PrimaryPointInfo {
            distance: self.info_ret1.get_distance(n_vec),
            reflectifity: self.info_ret1.get_reflectifity(),
            nir: (self.nir >> 8) as u8,
            signal: self.signal_ret_1,
        }
    }

    fn get_infos(&self, n_vec: u32) -> PointInfo<Self::Infos> {
        let primary = self.get_primary_infos(n_vec);
        PointInfo {
            channel_info: [
                PointChannelInfo {
                    distance: primary.distance,
                    reflectifity: primary.reflectifity,
                    signal: self.signal_ret_1,
                },
                PointChannelInfo {
                    distance: self.info_ret2.get_distance(n_vec),
                    reflectifity: self.info_ret2.get_reflectifity(),
                    signal: self.signal_ret_2,
                },
            ],
            nir: primary.nir,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable)]
pub struct SingleChannel {
    pub range_and_reserved: u32,
    pub reflectifity: u8,
    _reserved: u8,
    pub signal: u16,
    pub nir: u16,
    _reserved2: u16,
}

impl PointInfos for SingleChannel {
    type Signal = u16;
    type Infos = [PointChannelInfo<Self::Signal>; 1];

    fn get_primary_infos(&self, n_vec: u32) -> crate::PrimaryPointInfo<Self::Signal> {
        PrimaryPointInfo {
            distance: ((self.range_and_reserved & ((1 << 20) - 1)).saturating_sub(n_vec))
                .min(u16::MAX as _) as u16,
            reflectifity: self.reflectifity,
            nir: (self.nir >> 8) as u8,
            signal: self.signal,
        }
    }

    fn get_infos(&self, n_vec: u32) -> PointInfo<Self::Infos> {
        let primary = self.get_primary_infos(n_vec);
        PointInfo {
            nir: primary.nir,
            channel_info: [PointChannelInfo {
                distance: primary.distance,
                reflectifity: primary.reflectifity,
                signal: primary.signal,
            }],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable)]
pub struct LowDataChannel {
    pub distance_and_reserve: u16,
    pub reflectifity: u8,
    pub nir: u8,
}

impl PointInfos for LowDataChannel {
    type Signal = ();
    type Infos = [PointChannelInfo<Self::Signal>; 1];
    fn get_primary_infos(&self, n_vec: u32) -> crate::PrimaryPointInfo<Self::Signal> {
        PrimaryPointInfo {
            distance: (((self.distance_and_reserve.overflowing_mul(2).0) / 2) as u32 * 8)
                .saturating_sub(n_vec)
                .min(u16::MAX as _) as u16,
            reflectifity: self.reflectifity,
            nir: self.nir,
            signal: (),
        }
    }

    fn get_infos(&self, n_vec: u32) -> PointInfo<Self::Infos> {
        let primary = self.get_primary_infos(n_vec);
        PointInfo {
            nir: primary.nir,
            channel_info: [PointChannelInfo {
                distance: primary.distance,
                reflectifity: primary.reflectifity,
                signal: (),
            }],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable)]
pub struct RangeData {
    pub(crate) raw: u32,
}

impl RangeData {
    pub fn get_distance(&self, n_vec: u32) -> u16 {
        (self.raw & ((1 << 20) - 1))
            .saturating_sub(n_vec)
            .min(u16::MAX as _) as u16
    }

    pub fn get_reflectifity(&self) -> u8 {
        (self.raw >> 24) as u8
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
