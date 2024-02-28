use std::fmt::Debug;

use crate::{DualChannel, LowDataChannel, SingleChannel};

pub trait Profile: Clone + Copy + Send + Sync + 'static {
    type Array<T>: AsRef<[T]>;
    type Channel: Default + Copy + Debug + PointInfos + Send + Sync + 'static;
    type Channels: AsRef<[Self::Channel]> + Copy + Debug + Send + Sync + 'static;

    const COLUMNS: usize;
    const LAYERS: usize;

    fn initialize_channels() -> Self::Channels;
}

pub trait PointInfos {
    fn get_primary_infos_uncorrected(&self) -> PointInfo;
}

pub struct PointInfo {
    pub distance: u32,
}

#[derive(Clone, Copy)]
pub struct DualProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for DualProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Channel = DualChannel;
    type Channels = [Self::Channel; LAYERS];

    const COLUMNS: usize = COLUMNS;
    const LAYERS: usize = LAYERS;

    fn initialize_channels() -> Self::Channels {
        [Self::Channel::default(); LAYERS]
    }
}

#[derive(Clone, Copy)]
pub struct SingleProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for SingleProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Channel = SingleChannel;
    type Channels = [Self::Channel; LAYERS];

    const COLUMNS: usize = COLUMNS;
    const LAYERS: usize = LAYERS;

    fn initialize_channels() -> Self::Channels {
        [Self::Channel::default(); LAYERS]
    }
}

#[derive(Clone, Copy)]
pub struct LowDataProfile<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Profile for LowDataProfile<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Channel = LowDataChannel;
    type Channels = [Self::Channel; LAYERS];

    const COLUMNS: usize = COLUMNS;
    const LAYERS: usize = LAYERS;

    fn initialize_channels() -> Self::Channels {
        [Self::Channel::default(); LAYERS]
    }
}
