use std::fmt::Debug;

use crate::{Column, DualChannel, LowDataChannel, SingleChannel};

pub trait Profile: Copy + Send + Sync + 'static {
    type Array<T>: AsRef<[T]>;
    type Columns: AsRef<[Column<Self>]> + Clone + Send + Sync + 'static;
    type Channel: Default + Debug + PointInfos + Send + Sync + 'static;
    type Channels: AsRef<[Self::Channel]> + Debug + Send + Sync + 'static;

    const COLUMNS: usize;
    const LAYERS: usize;

    fn initialize_channels() -> Self::Channels;
    fn initialize_columns() -> Self::Columns;
}

pub trait PointInfos {
    type Infos: AsRef<[PointChannelInfo]>;
    fn get_primary_infos(&self, n_vec: u32) -> PrimaryPointInfo;
    fn get_infos(&self, n_vec: u32) -> PointInfo<Self::Infos>;
}

pub struct PointInfo<T> {
    pub channel_info: T,
    pub nir: u8,
}

pub struct PointChannelInfo {
    pub distance: u16,
    pub reflectifity: u8,
}

pub struct PrimaryPointInfo {
    pub distance: u16,
    pub reflectifity: u8,
    pub nir: u8,
}

#[derive(Clone, Copy)]
pub struct DualProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for DualProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
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

#[derive(Clone, Copy)]
pub struct SingleProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for SingleProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
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

#[derive(Clone, Copy)]
pub struct LowDataProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for LowDataProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
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
