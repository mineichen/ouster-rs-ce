use std::{any::Any, fmt::Debug};

use bytemuck::Zeroable;

use crate::Column;

mod dual;
mod dual_low;
mod low;
mod single;

pub use dual::*;
pub use dual_low::*;
pub use low::*;
pub use single::*;

pub trait Profile: Clone + Zeroable + Send + Sync + 'static {
    type Array<T>: AsRef<[T]>;
    type Header: Zeroable + Default + crate::PacketHeader + Clone;
    type Columns: AsRef<[Column<Self>]> + Clone + Zeroable + Send + Sync + 'static;
    type Channel: Default + Debug + PointInfos + Send + Sync + 'static;
    type Channels: AsRef<[Self::Channel]> + Zeroable + Debug + Send + Sync + 'static;

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
