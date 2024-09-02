use bytemuck::Zeroable;

use crate::{Column, OusterPacketHeaderSafety, Profile};

use super::{PointChannelInfo, PointInfo, PointInfos, PrimaryPointInfo};

#[derive(Clone, Copy, Zeroable)]
pub struct DualLowProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for DualLowProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Header = OusterPacketHeaderSafety;
    type Columns = [Column<Self>; COLUMNS];
    type Channel = DualLowChannel;
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
pub struct DualLowChannel {
    pub range_ret1: u16,
    pub reflect_ret_1: u8,
    pub nir: u8,
    pub range_ret2: u16,
    pub reflect_ret_2: u8,
}

impl PointInfos for DualLowChannel {
    type Signal = ();
    type Infos = [PointChannelInfo<Self::Signal>; 2];
    fn get_primary_infos(&self, n_vec: u32) -> crate::PrimaryPointInfo<Self::Signal> {
        PrimaryPointInfo {
            distance: (((self.range_ret1.overflowing_mul(2).0) / 2) as u32 * 8)
                .saturating_sub(n_vec)
                .min(u16::MAX as _) as u16,
            reflectifity: self.reflect_ret_1,
            nir: self.nir,
            signal: (),
        }
    }

    fn get_infos(&self, n_vec: u32) -> PointInfo<Self::Infos> {
        let primary = self.get_primary_infos(n_vec);
        PointInfo {
            channel_info: [
                PointChannelInfo {
                    distance: primary.distance,
                    reflectifity: primary.reflectifity,
                    signal: (),
                },
                PointChannelInfo {
                    distance: (((self.range_ret2.overflowing_mul(2).0) / 2) as u32 * 8)
                        .saturating_sub(n_vec)
                        .min(u16::MAX as _) as u16,
                    reflectifity: self.reflect_ret_2,
                    signal: (),
                },
            ],
            nir: primary.nir,
        }
    }
}
