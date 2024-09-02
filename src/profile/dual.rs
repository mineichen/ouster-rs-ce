use bytemuck::Zeroable;

use crate::{Column, OusterPacketHeader, Profile, RangeData};

use super::{PointChannelInfo, PointInfo, PointInfos, PrimaryPointInfo};

#[derive(Clone, Copy, Zeroable)]
pub struct DualProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for DualProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Header = OusterPacketHeader;
    type Columns = [Column<Self>; COLUMNS];
    type Channel = DualChannel;
    type Channels = [Self::Channel; LAYERS];

    const COLUMNS: usize = COLUMNS;
    const LAYERS: usize = LAYERS;

    fn initialize_channels() -> Self::Channels {
        [Self::Channel::default(); LAYERS]
    }
    fn initialize_columns() -> Self::Columns {
        [Column::<Self>::default(); COLUMNS]
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
