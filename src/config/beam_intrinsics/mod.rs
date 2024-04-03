use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeamIntrinsics {
    pub beam_altitude_angles: Vec<f32>,
    pub beam_azimuth_angles: Vec<f32>,
    pub lidar_origin_to_beam_origin_mm: f32,
    pub beam_to_lidar_transform: [f32; 16],
}

impl BeamIntrinsics {
    pub fn n_vec(&self) -> u32 {
        let offset_x = self.beam_to_lidar_transform[3];
        let offset_z = self.beam_to_lidar_transform[2 * 4 + 3];
        (offset_x * offset_x + offset_z * offset_z).sqrt().round() as u32
    }
}
