use std::{fmt::Debug, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{InvalidConfig, LidarProfile, Profile};

mod valid_window;

pub use valid_window::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LidarDataFormat {
    pub columns_per_packet: u8,
    pub pixels_per_column: u8,
    pub columns_per_frame: u16,
    pub pixel_shift_by_row: Box<[i8]>,
    pub column_window: (u16, u16),
    pub udp_profile_lidar: LidarProfile,
}

pub struct ValidLidarDataFormat<T> {
    pub columns_per_frame: u16,
    pub pixel_shift_by_row: Box<[i8]>,
    pub column_window: ValidWindow<T>,
    pub udp_profile_lidar: LidarProfile,
    phantom: PhantomData<T>,
}
impl<T: Profile> ValidLidarDataFormat<T> {
    pub fn calc_complete_cols_aligned(&self, alignment: usize) -> (usize, usize) {
        self.column_window
            .calc_complete_cols_aligned(&self.pixel_shift_by_row, alignment)
    }
}

impl<T: Profile> TryFrom<LidarDataFormat> for ValidLidarDataFormat<T> {
    type Error = InvalidConfig;

    fn try_from(value: LidarDataFormat) -> Result<Self, Self::Error> {
        if value.pixels_per_column as usize != T::LAYERS {
            return Err(InvalidConfig::new(format!(
                "Expected pixels_per_column to be {}, got {}",
                T::LAYERS,
                value.pixels_per_column
            )));
        }
        if value.columns_per_packet as usize != T::COLUMNS {
            return Err(InvalidConfig::new(format!(
                "Expected columns_per_packet to be {}, got {}",
                T::LAYERS,
                value.columns_per_packet
            )));
        }

        let column_window = ValidWindow::from(&value);

        Ok(ValidLidarDataFormat {
            columns_per_frame: value.columns_per_frame,
            pixel_shift_by_row: value.pixel_shift_by_row,
            column_window,
            udp_profile_lidar: value.udp_profile_lidar,
            phantom: PhantomData,
        })
    }
}
