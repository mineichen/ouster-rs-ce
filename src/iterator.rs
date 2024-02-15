use std::{f32::consts::PI, sync::Arc};

use crate::config::{OusterConfig, PolarPoint};

impl CartesianIterator<Arc<[(f32, f32)]>> {
    pub fn from_config(config: &OusterConfig) -> Self {
        let azimuth_roh_lut = config
            .beam_intrinsics
            .beam_azimuth_angles
            .iter()
            .zip(config.beam_intrinsics.beam_altitude_angles.iter())
            .map(|(azi, roh)| (-2. * PI * (azi / 360.), 2. * PI * (roh / 360.)))
            .collect::<Arc<_>>();

        let offset_x = config.beam_intrinsics.beam_to_lidar_transform[4 + 3];
        let offset_z = config.beam_intrinsics.beam_to_lidar_transform[2 * 4 + 3];
        Self::new(
            azimuth_roh_lut,
            config.config_params.lidar_mode.horizontal_resolution(),
            offset_x,
            offset_z,
        )
    }
}

#[derive(Clone)]
pub struct CartesianIterator<TSlice> {
    azimuth_alt: TSlice,
    azi_pos: usize,
    alt_pos: usize,
    translation: (f32, f32, f32),
    horizontal_resolution: u16,
    encoder_angle: f32,
    //per_row: PerRow,
    offset_x: f32,
}

impl<TSlice> CartesianIterator<TSlice>
where
    TSlice: AsRef<[(f32, f32)]>,
{
    fn new(azimuth_alt: TSlice, horizontal_resolution: u16, offset_x: f32, offset_z: f32) -> Self {
        assert!(!azimuth_alt.as_ref().is_empty());
        Self {
            horizontal_resolution,
            azimuth_alt,
            alt_pos: 0,
            azi_pos: 0,
            encoder_angle: 2. * PI,
            translation: (offset_x, 4.7411118e-6, offset_z),
            offset_x,
        }
    }
}

impl<TSlice> Iterator for CartesianIterator<TSlice>
where
    TSlice: AsRef<[(f32, f32)]>,
{
    type Item = PolarPoint;

    fn next(&mut self) -> Option<Self::Item> {
        let azi_alt = self.azimuth_alt.as_ref();

        if self.alt_pos < azi_alt.len() {
            let before = self.alt_pos;
            self.alt_pos += 1;
            let (azi, alt) = azi_alt[before];
            Some(PolarPoint {
                translation: self.translation,
                azimuth: self.encoder_angle + azi,
                roh: alt,
            })
        } else if self.azi_pos + 1 != self.horizontal_resolution as usize {
            self.azi_pos += 1;
            self.encoder_angle =
                2. * PI * (1. - (self.azi_pos as f32 / self.horizontal_resolution as f32));

            self.translation.0 = self.offset_x * self.encoder_angle.cos();
            self.translation.1 = self.offset_x * self.encoder_angle.sin();
            let (azi, alt) = azi_alt[0];
            self.alt_pos = 1;
            Some(PolarPoint {
                translation: self.translation,
                azimuth: self.encoder_angle + azi,
                roh: alt,
            })
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_all() {
        let x = CartesianIterator::new([(0.1, 0.2), (0.3, 0.4)], 2, 10., 15.).collect::<Vec<_>>();
        assert_eq!(
            4,
            x.iter()
                .zip([0.2, 0.4, 0.2, 0.4])
                .map(|(actual, expected)| {
                    assert!((actual.roh - expected).abs() < std::f32::EPSILON, "{x:?}");
                })
                .count()
        );
    }
}
