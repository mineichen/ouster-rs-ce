use std::fmt::Debug;

use crate::{DualChannel, LowDataChannel, SingleChannel};

pub trait Mode: Clone + Copy + Send + Sync + 'static {
    type Array<T>: AsRef<[T]>;
    type Channel: Default + Copy + Debug + PointInfos + Send + Sync + 'static;
}

pub trait PointInfos {
    fn get_primary_infos_uncorrected(&self) -> PointInfo;
}

pub struct PointInfo {
    pub distance: u32,
}

#[derive(Clone, Copy)]
pub struct DualMode<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Mode for DualMode<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Channel = DualChannel;
}

#[derive(Clone, Copy)]
pub struct SingleMode<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Mode for SingleMode<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Channel = SingleChannel;
}

#[derive(Clone, Copy)]
pub struct LowDataMode<const COLUMNS: usize, const LAYERS: usize>;
impl<const COLUMNS: usize, const LAYERS: usize> Mode for LowDataMode<COLUMNS, LAYERS> {
    type Array<T> = [T; COLUMNS];
    type Channel = LowDataChannel;
}
