use std::{future::Future, num::Saturating};

use crate::{OusterConfig, OusterPacket};

#[derive(Clone)]
struct AggregatorEntry<const COLUMNS: usize, const LAYERS: usize> {
    complete_buf: Box<[Box<OusterPacket<COLUMNS, LAYERS>>]>,
    complete: usize,
}

impl<const COLUMNS: usize, const LAYERS: usize> AggregatorEntry<COLUMNS, LAYERS> {
    fn new(required_packets: usize) -> Self {
        Self {
            complete_buf: (0..required_packets)
                .map(|_| Default::default())
                .collect::<Box<_>>(),
            complete: Default::default(),
        }
    }
}

/// Columns per package (usually 16)
pub struct Aggregator<const COLUMNS: usize, const LAYERS: usize> {
    measurements_per_rotation: usize,
    entries: [AggregatorEntry<COLUMNS, LAYERS>; 2],
    tmp: Box<OusterPacket<COLUMNS, LAYERS>>,
    completion_historgram: Vec<Saturating<u32>>,
    cur_measurement: u16,
}

impl<const COLUMNS: usize, const LAYERS: usize> Default for Aggregator<COLUMNS, LAYERS> {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl<const COLUMNS: usize, const LAYERS: usize> Aggregator<COLUMNS, LAYERS> {
    pub fn new(measurements_per_rotation: usize) -> Self {
        let required_packets = measurements_per_rotation / COLUMNS;
        let entry = AggregatorEntry::new(required_packets);
        Self {
            measurements_per_rotation,
            entries: [entry.clone(), entry],
            tmp: Default::default(),
            // +1 is to detect if more than the expected number of Packagers enters
            completion_historgram: vec![Saturating(0); required_packets + 1],
            cur_measurement: Default::default(),
        }
    }

    pub fn get_histogram(&self) -> impl Iterator<Item = u32> + '_ {
        self.completion_historgram.iter().map(|x| x.0)
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

    fn process_tmp(&mut self) -> Option<CompleteData<'_, COLUMNS, LAYERS>> {
        let idx = self.tmp.columns[0].channels_header.measurement_id as usize / 16;

        if self.cur_measurement < self.tmp.header.frame_id {
            self.entries.reverse();
            self.completion_historgram[0] +=
                (self.tmp.header.frame_id - self.cur_measurement - 1) as u32;
            self.completion_historgram[self.entries[0]
                .complete
                .min(self.entries[0].complete_buf.len())] += 1;
            self.entries[0].complete = 1;
            self.cur_measurement = self.tmp.header.frame_id;
            std::mem::swap(&mut self.entries[0].complete_buf[idx], &mut self.tmp);
            None
        } else {
            let entry_index = self.cur_measurement - self.tmp.header.frame_id;
            if let Some(entry) = self.entries.get_mut(entry_index as usize) {
                std::mem::swap(&mut entry.complete_buf[idx], &mut self.tmp);

                if entry.complete + 1 < self.measurements_per_rotation / COLUMNS {
                    entry.complete += 1;
                    None
                } else {
                    Some(CompleteData(&entry.complete_buf))
                }
            } else {
                None
            }
        }
    }
}

pub struct CompleteData<'a, const COLUMNS: usize, const LAYERS: usize>(
    &'a [Box<OusterPacket<COLUMNS, LAYERS>>],
);

impl<'a, const COLUMNS: usize, const LAYERS: usize> CompleteData<'a, COLUMNS, LAYERS> {
    pub fn iter(&self) -> impl Iterator<Item = &OusterPacket<COLUMNS, LAYERS>> {
        self.0.iter().map(AsRef::as_ref)
    }

    pub fn iter_points_flat(&self, config: &OusterConfig) -> impl Iterator<Item = u32> + '_ {
        let offset_x = config.beam_intrinsics.beam_to_lidar_transform[4 + 3];
        let offset_z = config.beam_intrinsics.beam_to_lidar_transform[2 * 4 + 3];
        let nvec = (offset_x * offset_x + offset_z * offset_z).sqrt().round() as u32;
        self.iter()
            .flat_map(|lidar_packet| lidar_packet.columns.iter())
            .flat_map(move |column| {
                column
                    .channels
                    .iter()
                    .map(move |point| point.info_ret1.get_distance() - nvec)
            })
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

    #[test]
    fn ignore_old_frame() {
        let mut aggregator = Aggregator::default();
        let mut packet = Dual64OusterPacket::default();
        packet.header.frame_id = 10;
        aggregator.put_data_value(packet);
        aggregator.put_data_value(Dual64OusterPacket::default());
    }

    #[test]
    fn with_unordered() {
        let mut input = (0..128).map(|i| {
            let mut x = Dual64OusterPacket::default();
            if i > 64 || i == 63 {
                x.header.frame_id = 1;
            }
            x
        });
        let mut aggregator = Aggregator::default();

        for (i, data) in (&mut input).take(64).enumerate() {
            assert!(aggregator.put_data_value(data).is_none(), "Item {i}");
        }
        aggregator
            .put_data_value(input.next().unwrap())
            .expect("Pointcloud should be complete");
        for i in (&mut input).take(62) {
            assert!(aggregator.put_data_value(i).is_none());
        }
        aggregator
            .put_data_value(input.next().unwrap())
            .expect("Pointcloud should be complete");
    }
}
