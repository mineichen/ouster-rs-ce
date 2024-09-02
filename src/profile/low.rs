use bytemuck::Zeroable;

use crate::{Column, OusterPacketHeader, Profile};

use super::{PointChannelInfo, PointInfo, PointInfos, PrimaryPointInfo};

#[derive(Clone, Copy, Zeroable)]
pub struct LowDataProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for LowDataProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Header = OusterPacketHeader;
    type Columns = [Column<Self>; COLUMNS];
    type Channel = LowDataChannel;
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
