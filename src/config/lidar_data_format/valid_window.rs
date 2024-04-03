use std::{fmt::Debug, marker::PhantomData};

use crate::{LidarDataFormat, Profile};

#[derive(Clone)]
pub struct ValidWindow<TProfile> {
    pub(crate) start_measurement_id: u16,
    pub(crate) required_measurements: usize,
    pub(crate) measurements_per_frame: u16,
    phantom: PhantomData<TProfile>,
}

impl<T> Debug for ValidWindow<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidWindow")
            .field("start_measurement_id", &self.start_measurement_id)
            .field("required_packets", &self.required_measurements)
            .field("typeof<T>", &std::any::type_name::<T>())
            .finish()
    }
}

impl<'a, TProfile: Profile> From<&'a LidarDataFormat> for ValidWindow<TProfile> {
    fn from(value: &'a LidarDataFormat) -> Self {
        Self::new(value.column_window, value.columns_per_frame)
    }
}

impl<TProfile: Profile> ValidWindow<TProfile> {
    pub fn new((column_from, column_to): (u16, u16), columns_per_frame: u16) -> Self {
        let start_measurement_id = (column_from / TProfile::COLUMNS as u16) as u16;
        let end_measurement_id = (column_to / TProfile::COLUMNS as u16) as u16;

        let required_measurements = (end_measurement_id
            + if column_from > column_to {
                (columns_per_frame as usize / TProfile::COLUMNS) as _
            } else {
                0
            })
            + if start_measurement_id == end_measurement_id {
                0
            } else {
                1
            }
            - start_measurement_id;
        Self {
            start_measurement_id,
            required_measurements: (required_measurements as _),
            measurements_per_frame: columns_per_frame / TProfile::COLUMNS as u16,
            phantom: PhantomData,
        }
    }

    pub const fn start_measurement_id(&self) -> u16 {
        self.start_measurement_id
    }

    pub const fn start(&self) -> usize {
        self.start_measurement_id as usize * TProfile::COLUMNS
    }

    pub const fn len(&self) -> usize {
        self.required_measurements * TProfile::COLUMNS
    }

    pub const fn end(&self) -> usize {
        (self.start_measurement_id as usize + self.required_measurements) * TProfile::COLUMNS
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

        let cut_start = min.unsigned_abs();

        let cut_len = self
            .len()
            .saturating_sub(max as usize)
            .saturating_sub(cut_start);
        let modulo = cut_len % alignment;
        let modulo_half = modulo / 2;

        let skip_first = cut_start + modulo_half;
        let take = cut_len - modulo;
        (skip_first, take)
    }
}

#[cfg(test)]
mod tests {
    use crate::{DualProfile, LowDataProfile, ValidWindow};

    type TestProfile = DualProfile<16, 128>;

    #[test]
    fn wrapping_end_in_same() {
        let window = ValidWindow::<LowDataProfile<16, 128>>::new((2, 0), 1024);
        assert_eq!(window.required_measurements, 1024 / 16);
    }
    #[test]
    fn wrapping_without_packet_overlap() {
        let window = ValidWindow::<LowDataProfile<16, 128>>::new((16, 0), 1024);
        assert_eq!(window.required_measurements, 1024 / 16);
    }

    #[test]
    fn complete_cols_wrapping() {
        let res = ValidWindow::<TestProfile>::new((33, 15), 1024)
            .calc_complete_cols_aligned(&[-1, 1], 16);
        assert_eq!((8, ((15 + 1024 - 33) / 16) * 16), res);
    }

    #[test]
    fn calc_17_remaining() {
        let res = ValidWindow::<TestProfile>::new((16, 159), 1024)
            .calc_complete_cols_aligned(&[-64, 63], 16);
        assert_eq!((64, 16), res);
    }

    #[test]
    fn calc_33_remaining() {
        let res = ValidWindow::<TestProfile>::new((16, 160), 1024)
            .calc_complete_cols_aligned(&[-64, 63], 16);
        assert_eq!((64, 32), res);
    }
    #[test]
    fn calc_32_remaining() {
        let res = ValidWindow::<TestProfile>::new((16, 160), 1024)
            .calc_complete_cols_aligned(&[-64, 64], 16);
        assert_eq!((64, 32), res);
    }
    #[test]
    fn calc_complete_cols_evenly_aligned() {
        let res = ValidWindow::<TestProfile>::new((16, 160), 1024)
            .calc_complete_cols_aligned(&[-64, 60], 16);
        assert_eq!((66, 32), res);
    }

    #[test]
    fn small_doesnt_panic() {
        let w = ValidWindow::<LowDataProfile<13, 128>>::new((0, 1), 1024);
        assert_eq!((32, 0), w.calc_complete_cols_aligned(&[32; 128], 16));
    }
}
