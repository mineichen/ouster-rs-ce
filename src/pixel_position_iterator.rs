use std::ops::RangeInclusive;

use crate::OusterConfig;

pub struct PixelPositionIterator<'a> {
    pixel_shifts: &'a [i8],

    last_col: usize,
    col: usize,
    row: usize,
    stride: usize,
}

impl<'a> PixelPositionIterator<'a> {
    pub fn from_config(config: &'a OusterConfig) -> Self {
        Self::new(
            &config.lidar_data_format.pixel_shift_by_row,
            (config.lidar_data_format.column_window.0 as usize)
                ..=(config.lidar_data_format.column_window.1 as usize),
        )
    }
    pub fn new(pixel_shifts: &'a [i8], col: RangeInclusive<usize>) -> Self {
        Self {
            pixel_shifts,
            last_col: *col.end(),
            row: 0,
            col: *col.start(),
            stride: col.count(),
        }
    }
}

impl<'a> Iterator for PixelPositionIterator<'a> {
    type Item = (usize, usize);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let (col, row, offset) = if let Some(&offset) = self.pixel_shifts.get(self.row) {
            self.row += 1;
            (self.col as isize, self.row - 1, offset as isize)
        } else if self.col < self.last_col {
            self.row = 1;
            self.col += 1;
            (self.col as isize, 0, self.pixel_shifts[0] as isize)
        } else {
            return None;
        };

        let col_shift = col + offset;

        Some((
            (col_shift + self.stride as isize) as usize % self.stride,
            row,
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn fill_all_fields() {
        let iter = PixelPositionIterator::new(&[1, -1, 3], 0..=3);

        let mut data = [0; 12];
        for (col, row) in iter {
            data[row + col * 3] = 1;
        }
        assert!(data.iter().all(|&x| x == 1));
    }

    #[test]
    fn simple() {
        let iter = PixelPositionIterator::new(&[1, -1], 0..=2);

        #[rustfmt::skip]
        assert_eq!(
            vec![(1, 0), (2, 1), 
                 (2, 0), (0, 1), 
                 (0, 0), (1, 1)],
            iter.inspect(|a| println!("{a:?}")).collect::<Vec<_>>()
        );
    }

    #[test]
    fn upper_overflow() {
        let iter = PixelPositionIterator::new(&[1], 0..=2);

        #[rustfmt::skip]
        assert_eq!(
            vec![(1, 0), 
                 (2, 0), 
                 (0, 0)],
            iter.inspect(|a| println!("{a:?}")).collect::<Vec<_>>()
        );
    }
}
