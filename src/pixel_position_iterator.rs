use crate::OusterConfig;

pub struct PixelPositionIterator<'a> {
    pixel_shifts: &'a [i8],
    total_cols: usize,
    col: usize,
    row: usize,
}

impl<'a> PixelPositionIterator<'a> {
    pub fn from_config(config: &'a OusterConfig) -> Self {
        Self {
            pixel_shifts: &config.lidar_data_format.pixel_shift_by_row,
            total_cols: config.lidar_data_format.columns_per_frame as _,
            row: 0,
            col: 0,
        }
    }
}

impl<'a> Iterator for PixelPositionIterator<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (col, row, offset) = if let Some(&offset) = self.pixel_shifts.get(self.row) {
            self.row += 1;
            (self.col as isize, self.row - 1, offset as isize)
        } else if self.col < self.total_cols - 1 {
            self.row = 1;
            self.col += 1;
            (self.col as isize, 0, self.pixel_shifts[0] as isize)
        } else {
            return None;
        };

        let col_shift = col + offset;

        Some((
            (col_shift + self.total_cols as isize) as usize % self.total_cols,
            row,
        ))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn fill_all_fields() {
        let iter = PixelPositionIterator {
            col: 0,
            row: 0,
            pixel_shifts: &[1, -1, 3],
            total_cols: 4,
        };

        let mut data = vec![0; 12];
        for (col, row) in iter {
            data[row + col * 3] = 1;
        }
        assert!(data.iter().all(|&x| x == 1));
    }

    #[test]
    fn simple() {
        let iter = PixelPositionIterator {
            col: 0,
            row: 0,
            pixel_shifts: &[1, -1],
            total_cols: 3,
        };

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
        let iter = PixelPositionIterator {
            col: 0,
            row: 0,
            pixel_shifts: &[1],
            total_cols: 3,
        };

        #[rustfmt::skip]
        assert_eq!(
            vec![(1, 0), 
                 (2, 0), 
                 (0, 0)],
            iter.inspect(|a| println!("{a:?}")).collect::<Vec<_>>()
        );
    }
}
