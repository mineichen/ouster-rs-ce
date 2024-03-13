use std::{borrow::Cow, ops::RangeInclusive, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OusterConfig {
    pub beam_intrinsics: BeamIntrinsics,
    pub config_params: ConfigParams,
    pub lidar_data_format: LidarDataFormat,
}

#[derive(Debug)]
pub struct PolarPoint {
    pub translation: (f32, f32, f32),
    pub azimuth: f32,
    pub roh: f32,
}

impl PolarPoint {
    pub fn calc_xyz(&self, distance: f32) -> (f32, f32, f32) {
        let x = distance * self.azimuth.cos() * self.roh.cos() + self.translation.0;
        let y = distance * self.azimuth.sin() * self.roh.cos() + self.translation.1;
        let z = distance * self.roh.sin() + self.translation.2;
        (x, y, z)
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

impl LidarDataFormat {
    pub fn shift_range(&self) -> RangeInclusive<i8> {
        let (min, max) = self
            .pixel_shift_by_row
            .into_iter()
            .fold((i8::MAX, i8::MIN), |(acc_min, acc_max), v| {
                (acc_min.min(*v), acc_max.max(*v))
            });
        min..=max
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeamIntrinsics {
    pub beam_altitude_angles: Vec<f32>,
    pub beam_azimuth_angles: Vec<f32>,
    pub lidar_origin_to_beam_origin_mm: f32,
    pub beam_to_lidar_transform: [f32; 16],
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigParams {
    pub lidar_mode: LidarMode,
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
