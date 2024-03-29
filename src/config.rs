use std::{borrow::Cow, marker::PhantomData, net::Ipv4Addr, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::Profile;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OusterConfig {
    pub beam_intrinsics: BeamIntrinsics,
    pub config_params: ConfigParams,
    pub lidar_data_format: LidarDataFormat,
}

/// Mustn't contain contradicting information like (window-size which doesnt't match Profile::Columns)
pub struct ValidOusterConfig<TProfile> {
    pub beam_intrinsics: BeamIntrinsics,
    pub config_params: ConfigParams,
    pub lidar_data_format: ValidLidarDataFormat<TProfile>,
    phantom: PhantomData<TProfile>,
}

impl<TProfile> ValidOusterConfig<TProfile> {
    pub fn n_vec(&self) -> u32 {
        let offset_x = self.beam_intrinsics.beam_to_lidar_transform[0 * 4 + 3];
        let offset_z = self.beam_intrinsics.beam_to_lidar_transform[2 * 4 + 3];
        (offset_x * offset_x + offset_z * offset_z).sqrt().round() as u32
    }
}

impl<T: Profile> TryFrom<OusterConfig> for ValidOusterConfig<T> {
    type Error = InvalidConfig;

    fn try_from(value: OusterConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            beam_intrinsics: value.beam_intrinsics,
            config_params: value.config_params,
            lidar_data_format: value.lidar_data_format.try_into()?,
            phantom: PhantomData,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{reason}")]
pub struct InvalidConfig {
    reason: String,
}

impl InvalidConfig {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

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

        let column_window = ValidWindow::new(value.column_window);

        Ok(ValidLidarDataFormat {
            columns_per_frame: value.columns_per_frame,
            pixel_shift_by_row: value.pixel_shift_by_row,
            column_window,
            udp_profile_lidar: value.udp_profile_lidar,
            phantom: PhantomData,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
#[non_exhaustive]
pub enum LidarProfile {
    SingleReturn,
    DualReturn,
    LowData,
}

impl Serialize for LidarProfile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            LidarProfile::SingleReturn => "RNG19_RFL8_SIG16_NIR16",
            LidarProfile::DualReturn => "RNG19_RFL8_SIG16_NIR16_DUAL",
            LidarProfile::LowData => "RNG15_RFL8_NIR8",
        })
    }
}

impl<'de> Deserialize<'de> for LidarProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let x = Cow::<str>::deserialize(deserializer)?;
        Self::from_str(&x).map_err(<D::Error as serde::de::Error>::custom)
    }
}

impl FromStr for LidarProfile {
    type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RNG19_RFL8_SIG16_NIR16" => Ok(Self::SingleReturn),
            "RNG15_RFL8_NIR8" => Ok(Self::LowData),
            "RNG19_RFL8_SIG16_NIR16_DUAL" => Ok(Self::DualReturn),
            s => Err(format!("Can't parse '{}'into LidarProfile", s).into()),
        }
    }
}

#[derive(Clone)]
pub struct ValidWindow<TProfile> {
    pub(crate) start_measurement_id: u16,
    pub(crate) required_packets: usize,
    phantom: PhantomData<TProfile>,
}

impl<TProfile: Profile> ValidWindow<TProfile> {
    pub fn new(column_window: (u16, u16)) -> Self {
        let start_measurement_id = (column_window.0 / TProfile::COLUMNS as u16) as _;
        let required_packets = ((column_window.1 + 1) as f32 / TProfile::COLUMNS as f32).ceil()
            as usize
            - start_measurement_id as usize;
        Self {
            start_measurement_id,
            required_packets,
            phantom: PhantomData,
        }
    }

    pub const fn start(&self) -> usize {
        self.start_measurement_id as usize * TProfile::COLUMNS
    }

    pub const fn len(&self) -> usize {
        self.required_packets * TProfile::COLUMNS
    }

    pub const fn end(&self) -> usize {
        (self.start_measurement_id as usize + self.required_packets - 1) * TProfile::COLUMNS
    }

    /// (skip_first, take)
    pub fn calc_complete_cols_aligned(
        &self,
        pixel_shift_by_row: &[i8],
        alignment: usize,
    ) -> (usize, usize) {
        let (min, max) = pixel_shift_by_row
            .iter()
            .fold((isize::MAX, isize::MIN), |(acc_min, acc_max), v| {
                (acc_min.min(*v as isize), acc_max.max(*v as isize))
            });

        let cut_start = min.abs() as usize;
        let cut_len = self.len() - max as usize - cut_start;
        let modulo = cut_len % alignment;
        let modulo_half = modulo / 2;

        let skip_first = cut_start + modulo_half;
        let take = cut_len - modulo;
        (skip_first, take)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeamIntrinsics {
    pub beam_altitude_angles: Vec<f32>,
    pub beam_azimuth_angles: Vec<f32>,
    pub lidar_origin_to_beam_origin_mm: f32,
    pub beam_to_lidar_transform: [f32; 16],
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigParams {
    pub azimuth_window: [u32; 2],
    pub lidar_mode: LidarMode,
    pub udp_dest: Ipv4Addr,
    pub udp_port_lidar: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LidarMode {
    #[serde(rename = "512x10")]
    Mode512x10,
    #[serde(rename = "512x20")]
    Mode512x20,
    #[serde(rename = "1024x10")]
    Mode1024x10,
    #[serde(rename = "1024x20")]
    Mode1024x20,
    #[serde(rename = "2048x10")]
    Mode2048x10,
}

impl LidarMode {
    pub fn horizontal_resolution(&self) -> u16 {
        match self {
            LidarMode::Mode512x10 | LidarMode::Mode512x20 => 512,
            LidarMode::Mode1024x10 | LidarMode::Mode1024x20 => 1024,
            LidarMode::Mode2048x10 => 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{DualProfile, ValidWindow};

    type TestProfile = DualProfile<16, 128>;

    #[test]
    fn calc_17_remaining() {
        let res =
            ValidWindow::<TestProfile>::new((16, 159)).calc_complete_cols_aligned(&[-64, 63], 16);
        assert_eq!((64, 16), res);
    }

    #[test]
    fn calc_33_remaining() {
        let res =
            ValidWindow::<TestProfile>::new((16, 160)).calc_complete_cols_aligned(&[-64, 63], 16);
        assert_eq!((64, 32), res);
    }
    #[test]
    fn calc_32_remaining() {
        let res =
            ValidWindow::<TestProfile>::new((16, 160)).calc_complete_cols_aligned(&[-64, 64], 16);
        assert_eq!((64, 32), res);
    }
    #[test]
    fn calc_complete_cols_evenly_aligned() {
        let res =
            ValidWindow::<TestProfile>::new((16, 160)).calc_complete_cols_aligned(&[-64, 60], 16);
        assert_eq!((66, 32), res);
    }
}
