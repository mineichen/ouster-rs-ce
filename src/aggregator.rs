use std::future::Future;

use crate::OusterPacket;

/// Columns per package (usually 16)
pub struct Aggregator<const COLUMNS: usize, const LAYERS: usize> {
    measurements_per_rotation: usize,
    complete_buf: Box<[Box<OusterPacket<COLUMNS, LAYERS>>]>,
    tmp: Box<OusterPacket<COLUMNS, LAYERS>>,
    complete: usize,
    cur_measurement: u16,
}

impl<const COLUMNS: usize, const LAYERS: usize> Default for Aggregator<COLUMNS, LAYERS> {
    fn default() -> Self {
        Self::new(1024)
    }
}

pub struct CompleteData<'a, const COLUMNS: usize, const LAYERS: usize>(
    &'a [Box<OusterPacket<COLUMNS, LAYERS>>],
);

impl<'a, const COLUMNS: usize, const LAYERS: usize> CompleteData<'a, COLUMNS, LAYERS> {
    pub fn iter(&self) -> impl Iterator<Item = &OusterPacket<COLUMNS, LAYERS>> {
        self.0.iter().map(AsRef::as_ref)
    }
}

impl<const COLUMNS: usize, const LAYERS: usize> Aggregator<COLUMNS, LAYERS> {
    pub fn new(measurements_per_rotation: usize) -> Self {
        let required_packets = measurements_per_rotation / COLUMNS;
        Self {
            measurements_per_rotation,
            complete_buf: (0..required_packets)
                .map(|_| Default::default())
                .collect::<Box<_>>(),
            tmp: Default::default(),
            complete: Default::default(),
            cur_measurement: Default::default(),
        }
    }

    pub fn put_data_value(
        &mut self,
        data: OusterPacket<COLUMNS, LAYERS>,
    ) -> Option<CompleteData<'_, COLUMNS, LAYERS>> {
        *self.tmp.as_mut() = data;
        self.process_tmp()
    }

    pub async fn put_data<TFut: Future<Output = std::io::Result<()>>>(
        &mut self,
        operator: impl FnOnce(&mut OusterPacket<COLUMNS, LAYERS>) -> TFut,
    ) -> std::io::Result<Option<CompleteData<'_, COLUMNS, LAYERS>>> {
        operator(self.tmp.as_mut()).await?;
        Ok(self.process_tmp())
    }
    pub fn put_data_sync(
        &mut self,
        operator: impl FnOnce(&mut OusterPacket<COLUMNS, LAYERS>) -> std::io::Result<()>,
    ) -> std::io::Result<Option<CompleteData<'_, COLUMNS, LAYERS>>> {
        operator(self.tmp.as_mut())?;
        Ok(self.process_tmp())
    }

    pub fn process_tmp(&mut self) -> Option<CompleteData<'_, COLUMNS, LAYERS>> {
        let idx = self.tmp.columns[0].channels_header.measurement_id as usize / 16;
        // Todo: Change API to avoid MEMCPY
        self.complete_buf[idx] = self.tmp.clone();

        if self.cur_measurement != self.tmp.header.frame_id {
            self.complete = 1;
            self.cur_measurement = self.tmp.header.frame_id;
            None
        } else if self.complete + 1 < self.measurements_per_rotation / COLUMNS {
            self.complete += 1;
            None
        } else {
            Some(CompleteData(&self.complete_buf))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Dual64OusterPacket;

    use super::Aggregator;

    #[test]
    fn without_missing() {
        let mut input = (0..64).map(|_| Dual64OusterPacket::default());
        let mut aggregator = Aggregator::default();

        for i in (&mut input).take(63) {
            assert!(aggregator.put_data_value(i).is_none());
        }
        aggregator
            .put_data_value(input.next().unwrap())
            .expect("Pointcloud should be complete");
    }
}
