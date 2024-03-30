use crate::{Profile, ValidLidarDataFormat};

pub struct PixelPositionIterator<'a> {
    pixel_shifts: &'a [i8],
    col: usize,
    row: usize,
    col_len: usize,
}

impl<'a> PixelPositionIterator<'a> {
    pub fn from_config<TProfile: Profile>(config: &'a ValidLidarDataFormat<TProfile>) -> Self {
        let window = &config.column_window;

        Self::new(&config.pixel_shift_by_row, window.len())
    }
    pub fn new(pixel_shifts: &'a [i8], col_len: usize) -> Self {
        Self {
            pixel_shifts,
            row: if col_len == 0 { usize::MAX } else { 0 },
            col: if col_len == 0 { usize::MAX } else { 0 },
            col_len: col_len.max(1),
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
        let iter = PixelPositionIterator::new(&[1, -1, 3], 4);

        let mut data = [0; 12];
        for (col, row) in iter {
            println!("{col} {row}");
            data[row + col * 3] = 1;
        }
        assert!(data.iter().all(|&x| x == 1), "{data:?}");
    }

    #[test]
    fn simple() {
        let iter = PixelPositionIterator::new(&[1, -1], 3);

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
        let iter = PixelPositionIterator::new(&[1], 3);

        #[rustfmt::skip]
        assert_eq!(
            vec![(1, 0),
                 (2, 0),
                 (0, 0)],
            iter.inspect(|a| println!("{a:?}")).collect::<Vec<_>>()
        );
    }
    #[test]
    fn col_len_zero_returns_empty_iterator() {
        assert_eq!(0, PixelPositionIterator::new(&[1], 0).count());
    }
}
