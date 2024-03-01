use std::{any::Any, fmt::Debug};

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
    type Signal: Any;
    type Infos: AsRef<[PointChannelInfo<Self::Signal>]>;
    fn get_primary_infos(&self, n_vec: u32) -> PrimaryPointInfo<Self::Signal>;
    fn get_infos(&self, n_vec: u32) -> PointInfo<Self::Infos>;
}

pub struct PointInfo<T> {
    pub channel_info: T,
    pub nir: u8,
}

pub struct PointChannelInfo<TSignal> {
    pub distance: u16,
    pub reflectifity: u8,
    pub signal: TSignal,
}

impl<TSignal: Any> PointChannelInfo<TSignal> {
    pub fn unwrap_signal(&self) -> u16 {
        if let Some(x) = <dyn std::any::Any>::downcast_ref::<u16>(&self.signal) {
            *x
        } else {
            panic!(
                "Signal was unwrapped, but there was no signal: {}",
                std::any::type_name::<TSignal>()
            );
        }
    }
}

pub struct PrimaryPointInfo<TSignal: Any> {
    pub distance: u16,
    pub reflectifity: u8,
    pub nir: u8,
    pub signal: TSignal,
}

impl<TSignal: Any> PrimaryPointInfo<TSignal> {
    pub fn unwrap_signal(&self) -> u16 {
        if let Some(x) = <dyn std::any::Any>::downcast_ref::<u16>(&self.signal) {
            *x
        } else {
            panic!(
                "Signal was unwrapped, but there was no signal: {}",
                std::any::type_name::<TSignal>()
            );
        }
    }
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
