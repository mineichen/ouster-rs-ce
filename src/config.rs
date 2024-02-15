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
    pub pixels_per_column: u8,
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
