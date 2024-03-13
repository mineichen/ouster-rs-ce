use std::{f32::consts::PI, marker::PhantomData, sync::Arc};

use crate::{Profile, ValidOusterConfig, ValidWindow};

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

impl<TProfile: Profile> CartesianIterator<TProfile, Arc<[(f32, f32)]>> {
    pub fn new_cheap_cloneable_from_config(config: &ValidOusterConfig<TProfile>) -> Self {
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
            config.lidar_data_format.columns_per_frame,
            config.lidar_data_format.column_window.clone(),
            offset_x,
            offset_z,
        )
    }
}

#[derive(Clone)]
pub struct CartesianIterator<TProfile, TSlice> {
    azimuth_alt: TSlice,
    azi_pos: usize,
    alt_pos: usize,
    cols_per_frame: u16,
    translation: (f32, f32, f32),
    cols: ValidWindow<TProfile>,
    encoder_angle: f32,
    offset_x: f32,
    phantom: PhantomData<TProfile>,
}

impl<TProfile: Profile, TSlice> CartesianIterator<TProfile, TSlice>
where
    TSlice: AsRef<[(f32, f32)]>,
{
    fn new(
        azimuth_alt: TSlice,
        cols_per_frame: u16,
        cols: ValidWindow<TProfile>,
        offset_x: f32,
        offset_z: f32,
    ) -> Self {
        assert!(!azimuth_alt.as_ref().is_empty());

        let azi_pos = cols.start();
        let encoder_angle = 2. * PI * (1. - (azi_pos as f32 / cols_per_frame as f32));
        Self {
            azimuth_alt,
            azi_pos,
            alt_pos: 0,
            cols_per_frame,
            translation: (offset_x, 4.7411118e-6, offset_z),
            cols,
            encoder_angle,
            offset_x,
            phantom: PhantomData,
        }
    }
}

impl<TProfile: Profile, TSlice> Iterator for CartesianIterator<TProfile, TSlice>
where
    TSlice: AsRef<[(f32, f32)]>,
{
    type Item = PolarPoint;

    #[inline(always)]
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
        } else if self.azi_pos != self.cols.end() {
            self.azi_pos += 1;
            self.encoder_angle =
                2. * PI * (1. - (self.azi_pos as f32 / self.cols_per_frame as f32));

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
    use crate::DualProfile;

    use super::*;

    #[test]
    fn iter_all() {
        let x = CartesianIterator::new(
            [(0.1, 0.2), (0.3, 0.4)],
            2,
            ValidWindow::<DualProfile<1, 3>>::new((0, 1)),
            10.,
            15.,
        )
        .collect::<Vec<_>>();
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
