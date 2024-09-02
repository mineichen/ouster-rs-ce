use bytemuck::Zeroable;

use crate::{Column, OusterPacketHeader, Profile};

use super::{PointChannelInfo, PointInfo, PointInfos, PrimaryPointInfo};

#[derive(Clone, Copy, Zeroable)]
pub struct SingleProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for SingleProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Header = OusterPacketHeader;
    type Columns = [Column<Self>; COLUMNS];
    type Channel = SingleChannel;
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
