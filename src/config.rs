use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OusterSettings {
    pub beam_intrinsics: BeamIntrinsicsJsonFormat,
    pub config_params: ConfigParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeamIntrinsicsJsonFormat {
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
