use std::ops::RangeInclusive;

use crate::OusterConfig;

pub struct PixelPositionIterator<'a> {
    pixel_shifts: &'a [i8],
    col: usize,
    row: usize,
    col_len: usize,
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
        let len = col.end() - col.start() + 1;
        Self {
            pixel_shifts,
            row: 0,
            col: 0,
            col_len: len,
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
        } else if self.col < self.col_len - 1 {
            self.row = 1;
            self.col += 1;
            (self.col as isize, 0, self.pixel_shifts[0] as isize)
        } else {
            return None;
        };

        let col_shift = col + offset;

        Some((
            (col_shift + self.col_len as isize) as usize % self.col_len,
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
            println!("{col} {row}");
            data[row + col * 3] = 1;
        }
        assert!(data.iter().all(|&x| x == 1), "{data:?}");
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
