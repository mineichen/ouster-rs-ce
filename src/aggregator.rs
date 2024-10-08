use std::{num::Saturating, sync::Arc};

use bytemuck::Zeroable;

use crate::{
    profile::Profile, OusterPacket, PacketHeader, PointInfo, PointInfos, PrimaryPointInfo,
    ValidOperationConfig, ValidWindow,
};

#[derive(Clone)]
struct AggregatorEntry<TProfile: Profile> {
    frame_id: u16,
    complete_buf: Box<[Box<OusterPacket<TProfile>>]>,
    missing_packet_histogram: u128,
    count_packets: usize,
}

impl<TProfile: Profile> AggregatorEntry<TProfile> {
    fn new(required_packets: usize) -> Self {
        debug_assert!(required_packets <= 128);
        Self {
            frame_id: 0,
            complete_buf: (0..required_packets)
                .map(|_| Default::default())
                .collect::<Box<_>>(),
            missing_packet_histogram: 0,
            count_packets: Default::default(),
        }
    }
}

pub struct Aggregator<TProfile: Profile> {
    start_measurement_id: u16,
    captured_cols_per_rotation: usize,
    total_measurements_per_frame: u16,
    entry_active: AggregatorEntry<TProfile>,
    entry_other: AggregatorEntry<TProfile>,
    entry_out: Arc<AggregatorEntry<TProfile>>,
    tmp: Box<OusterPacket<TProfile>>,
    completion_historgram: Vec<Saturating<u32>>,
    missing_packets: Vec<Saturating<u32>>,
    dropped_packets: Saturating<u32>,
}

#[derive(Debug)]
pub struct AggregatorStatistics {
    pub completion_historgram: Vec<u32>,
    pub dropped_frames: u32,
    pub missing_packets: Vec<u32>,
}

impl<TProfile: Profile> Aggregator<TProfile> {
    pub fn new(valid_config: &ValidWindow<TProfile>) -> Self {
        let required_measurements = valid_config.required_measurements;
        Self {
            start_measurement_id: valid_config.start_measurement_id(),
            captured_cols_per_rotation: required_measurements * TProfile::COLUMNS,
            total_measurements_per_frame: valid_config.measurements_per_frame,
            entry_active: AggregatorEntry::new(required_measurements),
            entry_other: AggregatorEntry::new(required_measurements),
            entry_out: Arc::new(AggregatorEntry::new(required_measurements)),
            tmp: Default::default(),
            // +2 is to detect if more than the expected number of Packagers enters
            // Example required_packages=2 [none, one_package, two_packages, more]
            completion_historgram: vec![Saturating(0); required_measurements + 2],
            missing_packets: vec![Saturating(0); required_measurements],
            dropped_packets: Saturating(0),
        }
    }

    pub fn get_histogram(&self) -> Vec<u32> {
        let mut r = self
            .completion_historgram
            .iter()
            .map(|x| x.0)
            .collect::<Vec<_>>();

        r[self
            .entry_active
            .count_packets
            .min(self.missing_packets.len())] += 1;

        r
    }
    pub fn get_statistics(&self) -> AggregatorStatistics {
        AggregatorStatistics {
            completion_historgram: self.get_histogram(),
            dropped_frames: self.dropped_packets.0,
            missing_packets: self.missing_packets.iter().map(|x| x.0).collect::<Vec<_>>(),
        }
    }

    pub fn put_data_value(
        &mut self,
        data: OusterPacket<TProfile>,
    ) -> Option<CompleteData<TProfile>> {
        *self.tmp.as_mut() = data;
        self.process_tmp()
    }

    pub fn next_buffer(&mut self) -> &mut [u8] {
        let tmp: &mut OusterPacket<TProfile> = &mut self.tmp;
        unsafe {
            std::slice::from_raw_parts_mut(
                std::ptr::from_mut(tmp) as *mut u8,
                std::mem::size_of::<OusterPacket<TProfile>>(),
            )
        }
    }

    pub fn put_data_sync(
        &mut self,
        operator: impl FnOnce(&mut OusterPacket<TProfile>) -> std::io::Result<()>,
    ) -> std::io::Result<Option<CompleteData<TProfile>>> {
        operator(self.tmp.as_mut())?;
        Ok(self.process_tmp())
    }

    pub fn process_tmp(&mut self) -> Option<CompleteData<TProfile>> {
        let idx = {
            let pos = self.tmp.columns.as_ref()[0].channels_header.measurement_id
                / TProfile::COLUMNS as u16;
            let idx = if pos < self.start_measurement_id {
                pos + self.total_measurements_per_frame - self.start_measurement_id
            } else {
                pos - self.start_measurement_id
            };
            idx
        } as usize;

        if idx >= self.entry_active.complete_buf.len() {
            return None;
        }

        if self.entry_active.frame_id == self.tmp.header.frame_id() {
            std::mem::swap(&mut self.entry_active.complete_buf[idx], &mut self.tmp);
            self.entry_active.count_packets += 1;
            self.entry_active.missing_packet_histogram |= 1 << idx;
            None
        } else if self.entry_other.frame_id != self.tmp.header.frame_id() {
            self.entry_other.frame_id = self.tmp.header.frame_id();
            std::mem::swap(&mut self.entry_other.complete_buf[idx], &mut self.tmp);
            self.dropped_packets += self.entry_other.count_packets as u32;
            self.entry_other.count_packets = 1;
            self.entry_other.missing_packet_histogram = 1 << idx;
            None
        } else {
            self.entry_other.missing_packet_histogram |= 1 << idx;
            self.entry_other.count_packets += 1;
            std::mem::swap(&mut self.entry_other.complete_buf[idx], &mut self.tmp);
            // Finish delayed so out of order UDP Packets are still assigned
            if self.entry_other.count_packets == 10 {
                // Always output for now
                let out = Arc::make_mut(&mut self.entry_out);
                out.count_packets = 0;
                out.missing_packet_histogram = 0;

                std::mem::swap(out, &mut self.entry_active);
                std::mem::swap(&mut self.entry_active, &mut self.entry_other);

                // Statistics
                let result = if out.count_packets != 0 {
                    let last_index = self.completion_historgram.len() - 1;
                    self.completion_historgram[out.count_packets.min(last_index)] += 1;

                    let mut hist = out.missing_packet_histogram;
                    for x in 0..(self.captured_cols_per_rotation / TProfile::COLUMNS) {
                        if hist & 1 == 0 {
                            *out.complete_buf[x] = OusterPacket::zeroed();
                            self.missing_packets[x] += 1;
                        }
                        hist >>= 1;
                    }
                    Some(CompleteData(self.entry_out.clone()))
                } else {
                    None
                };

                return result;
            }
            None
        }
    }
}

pub struct CompleteData<TProfile: Profile>(Arc<AggregatorEntry<TProfile>>);

impl<TProfile: Profile> CompleteData<TProfile> {
    pub fn iter(&self) -> impl Iterator<Item = &OusterPacket<TProfile>> {
        self.0.complete_buf.iter().map(AsRef::as_ref)
    }

    pub fn is_empty(&self) -> bool {
        self.0.count_packets == 0
    }

    pub fn len(&self) -> usize {
        self.0.count_packets
    }

    pub fn statistics(&self) -> u128 {
        self.0.missing_packet_histogram
    }

    pub fn iter_flat<'a, T>(
        &'a self,
        config: &ValidOperationConfig<TProfile>,
        mut map: impl FnMut(&<TProfile as Profile>::Channel, u32) -> T + 'a,
    ) -> impl Iterator<Item = T> + '_ {
        let n_vec = config.n_vec();
        self.iter()
            .flat_map(|lidar_packet| lidar_packet.columns.as_ref().iter())
            .flat_map(move |column| column.channels.as_ref().iter())
            .map(move |x| map(x, n_vec))
    }

    pub fn iter_infos(
        &self,
        config: &ValidOperationConfig<TProfile>,
    ) -> impl Iterator<Item = PointInfo<<TProfile::Channel as PointInfos>::Infos>> + '_ {
        self.iter_flat(config, |point, nvec| point.get_infos(nvec))
    }

    pub fn iter_infos_primary(
        &self,
        config: &ValidOperationConfig<TProfile>,
    ) -> impl Iterator<Item = PrimaryPointInfo<<TProfile::Channel as PointInfos>::Signal>> + '_
    {
        self.iter_flat(config, |point, nvec| point.get_primary_infos(nvec))
    }

    // get_unchecked() didn't improve performance
    // the compiler optimized it out during inline. inline(always) makes sure optimization can be made
    #[inline(always)]
    pub fn get_row_first_infos_primary(
        &self,
        index: usize,
        n_vec: u32,
    ) -> PrimaryPointInfo<<TProfile::Channel as PointInfos>::Signal> {
        let column_idx = index / TProfile::LAYERS;
        let column_idx_outer = column_idx % TProfile::COLUMNS;
        let column_idx_inner = column_idx / TProfile::COLUMNS;
        let row_idx = index % TProfile::LAYERS;
        let outer = &self.0.complete_buf[column_idx_inner];
        let column_inner = &outer.columns.as_ref()[column_idx_outer];
        column_inner.channels.as_ref()[row_idx].get_primary_infos(n_vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dual64OusterPacket, DualProfile, Profile, ValidWindow};

    use super::Aggregator;

    #[test]
    fn bellow_start_measurement_id() {
        test_window((32, 0));
    }

    #[test]
    fn without_missing() {
        test_window((0, 1023));
    }

    #[test]
    fn high_start_measurement_id() {
        test_window((740, 240));
    }

    fn test_window(window: (u16, u16)) {
        let valid_win = ValidWindow::new(window, 1024);
        let mut input = (0..).map(|i| {
            let mut x = Dual64OusterPacket::default();
            x.header.frame_id = i / 64;
            x.columns[0].channels_header.measurement_id = ((i + valid_win.start_measurement_id())
                % 64)
                * DualProfile::<16, 128>::COLUMNS as u16;
            for col in x.columns.iter_mut() {
                for ch in col.channels.iter_mut() {
                    ch.info_ret1.raw = 1;
                }
            }
            x
        });
        let mut aggregator = Aggregator::new(&valid_win);

        for i in (&mut input).take(63 + 10) {
            assert!(aggregator.put_data_value(i).is_none());
        }
        let pc = aggregator
            .put_data_value(input.next().unwrap())
            .expect("Pointcloud should be complete");
        assert!(pc.iter().all(|x| {
            x.columns
                .iter()
                .all(|x| x.channels.iter().all(|f| f.info_ret1.raw == 1))
        }));
    }

    #[test]
    fn ignore_old_frame() {
        let mut aggregator = Aggregator::new(&ValidWindow::new((0, 1023), 1024));
        let mut packet = Dual64OusterPacket::default();
        packet.header.frame_id = 10;
        aggregator.put_data_value(packet);
        aggregator.put_data_value(Dual64OusterPacket::default());
    }

    #[test]
    fn with_unordered() {
        let mut input = (0u32..).map(|i| {
            let mut x = Dual64OusterPacket::default();

            x.header.frame_id = match i {
                0..=62 => 0,
                63 => 1,
                64 => 0,
                65..=127 => 1,
                128.. => 2,
            };

            x
        });
        let mut aggregator = Aggregator::new(&ValidWindow::new((0, 1023), 1024));

        for (i, data) in (&mut input).take(64 + 9).enumerate() {
            assert!(aggregator.put_data_value(data).is_none(), "Item {i}");
        }
        aggregator
            .put_data_value(input.next().unwrap())
            .expect("Pointcloud should be complete");
        for i in (&mut input).take(63) {
            assert!(aggregator.put_data_value(i).is_none());
        }
        aggregator
            .put_data_value(input.next().unwrap())
            .expect("Pointcloud should be complete");
        let hist = aggregator.get_histogram();
        assert_eq!(0, hist[0], "{:?}", hist);
        assert_eq!(2, hist[64], "{:?}", hist);
    }
}
