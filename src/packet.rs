use std::sync::Arc;

pub type Dual128OusterPacket = OusterPacket<16, 128>;
pub type Dual64OusterPacket = OusterPacket<16, 64>;

#[repr(C)]
#[derive(Debug)]
pub struct OusterPacket<const TCOLUMNS: usize, const TCHANNELS: usize> {
    header: OusterPacketHeader,
    blocks: [Column<TCHANNELS>; TCOLUMNS],
    reserved: [u32; 8],
}

#[repr(C)]
#[derive(Debug, Default)]
struct OusterPacketHeader {
    packet_type: u16,
    frame_id: u16,
    init_id_part1: u16,
    init_id_part2: u8,
    serial_no_1: u8,
    serial_no_2: u32,
    _reserved_1: u32,
    shutdown_countdown: u8,
    shot_limiting_countdown: u8,
    shutdown_status_and_reserve: u8,
    shot_limiting_status_and_reserve: u8,
    _reserved_2: [u32; 3],
}

impl<const TCOLUMNS: usize, const TCHANNELS: usize> Default for OusterPacket<TCOLUMNS, TCHANNELS> {
    fn default() -> Self {
        Self {
            header: Default::default(),
            blocks: [Default::default(); TCOLUMNS],
            reserved: [0; 8],
        }
    }
}
impl<const TCOLUMNS: usize, const TCHANNELS: usize> OusterPacket<TCOLUMNS, TCHANNELS> {
    // Not yet aware of Endianness... The buffer needs to be modified in that case and data_accessors of irregular bitsizes have to be adapted too
    // mut allows to implement this in the future without breaking changes
    #[cfg(target_endian = "little")]
    pub fn from_aligned_memory(buffer: &[u8]) -> &Self {
        if (buffer.as_ptr()) as usize % 32 != 0 {
            panic!("Buffer has to be aligned");
        }

        unsafe { &*(buffer.as_ptr() as *const Self) }
    }

    pub fn from_maybe_unaligned(buffer: &[u8]) -> Arc<Self> {
        let mut inner = Self::default();
        let s = std::mem::size_of::<Self>();
        {
            let inner_ptr: *mut u8 = (&mut inner) as *mut Self as _;
            let as_buf = unsafe { std::slice::from_raw_parts_mut(inner_ptr, s) };
            as_buf.copy_from_slice(buffer);
        }
        Arc::new(inner)
    }

    pub fn iter_cartesian(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        [].into_iter()
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Column<const TCHANNELS: usize> {
    channels_header: ChannelsHeader,
    channels: [Channel; TCHANNELS],
}

impl<const TCHANNELS: usize> Default for Column<TCHANNELS> {
    fn default() -> Self {
        Self {
            channels_header: ChannelsHeader::default(),
            channels: [Channel::default(); TCHANNELS],
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct ChannelsHeader {
    // Single u64 would force ChannelsHeader to be 64bit aligned
    timestamp_a: u32,
    timestamp_b: u32,
    measurement_id: u16,
    status_and_reserve: u16,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct Channel {
    info_ret1: RangeData,
    info_ret2: RangeData,
    signal_ret_1: u16,
    signal_ret_2: u16,
    nir: u16,
    _reserved: u16,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct RangeData {
    raw: u32,
}

impl RangeData {
    fn get_distance(&self) -> f32 {
        (self.raw & ((1 << 20) - 1)) as f32
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use image::{ImageEncoder, Luma};
    use pcap::Capture;
    use pcd_rs::{DataKind, PcdSerialize, WriterInit};
    use std::{f32::consts::PI, io::Cursor, path::PathBuf};

    use super::*;

    const UDP_HEADER_SIZE: usize = 42;

    #[test]
    fn assert_correct_structsize() {
        assert_eq!(256 / 8, std::mem::size_of::<OusterPacketHeader>());
        assert_eq!(96 / 8, std::mem::size_of::<ChannelsHeader>());
        assert_eq!(128 / 8, std::mem::size_of::<super::Channel>());
        assert_eq!(32 / 8, std::mem::size_of::<super::RangeData>());
        assert_eq!(33024, std::mem::size_of::<Dual128OusterPacket>());
    }

    #[test]
    fn ouster_pcd_64() -> Result<(), Box<dyn std::error::Error>> {
        ouster_pcd_converter::<64>(
            "OS-0-64-U02_v3.0.1_1024x10_20230510_135903.json",
            "OS-0-64-U02_v3.0.1_1024x10_20230510_135903-000.pcap",
        )
    }

    #[test]
    fn ouster_pcd_128() -> Result<(), Box<dyn std::error::Error>> {
        ouster_pcd_converter::<128>(
            "OS-0-128_v3.0.1_1024x10_20230510_134250.json",
            "OS-0-128_v3.0.1_1024x10_20230510_134250-000.pcap",
        )
    }

    fn ouster_pcd_converter<const LAYERS: usize>(
        test_json_path: &str,
        test_pcap_file: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load pcap file
        let home = PathBuf::from(env!("HOME"));
        let test_files = home.join("Downloads");
        let target = home.join("demo.pcd");

        let data = std::fs::read(test_files.join(test_json_path))?;
        let config: crate::config::OusterSettings = serde_json::from_slice(&data)?;
        let mut cap = Capture::from_file(test_files.join(test_pcap_file))?;
        cap.filter("udp", true)?;

        let mut cnt = 0;
        let mut min = f32::MAX;
        let mut max = 0f32;
        let mut skip_complete = 100;
        const SCAN_WIDTH: u16 = 1024;

        //const CAPTURE_POINTS: usize = 70974464;
        const CAPTURE_POINTS: usize = 151072;
        let mut buf = Vec::new();
        let mut writer = WriterInit {
            width: CAPTURE_POINTS as _,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: None,
        }
        .build_from_writer(Cursor::new(&mut buf))?;

        const REQUIRED_PACKETS: usize = 1024 / 16;
        let mut complete_buf = (0..REQUIRED_PACKETS)
            .map(|_| Box::new([Column::<LAYERS>::default(); 16]))
            .collect::<Box<_>>();
        let mut cur_measurement = 0;
        let mut complete = 0;

        let offset_x = config.beam_intrinsics.beam_to_lidar_transform[0 * 4 + 3];
        let offset_z = config.beam_intrinsics.beam_to_lidar_transform[2 * 4 + 3];
        let nvec = (offset_x * offset_x + offset_z * offset_z).sqrt();
        let azimuth_roh_lut = config
            .beam_intrinsics
            .beam_azimuth_angles
            .iter()
            .zip(config.beam_intrinsics.beam_altitude_angles.iter())
            .map(|(azi, roh)| (-2. * PI * (azi / 360.), 2. * PI * (roh / 360.)))
            .collect::<Box<_>>();

        println!("Ready now");

        // let total_lut = config
        //     .beam_intrinsics
        //     .beam_altitude_angles
        //     .iter()
        //     .flat_map(|rows| azimuth_roh_lut);
        let mut image = vec![0u8; 1024 * LAYERS];

        while let Ok(packet) = cap.next_packet() {
            let slice = &packet.data[UDP_HEADER_SIZE..];
            if slice.len() != std::mem::size_of::<OusterPacket<16, LAYERS>>() {
                continue;
            }
            let lidar_packet = OusterPacket::<16, LAYERS>::from_maybe_unaligned(slice);
            *complete_buf[lidar_packet.blocks[0].channels_header.measurement_id as usize / 16] =
                lidar_packet.blocks;

            if cur_measurement != lidar_packet.header.frame_id {
                complete = 1;
                cur_measurement = lidar_packet.header.frame_id;
            } else if complete + 1 < REQUIRED_PACKETS {
                complete += 1;
            } else {
                cur_measurement += 1;
                complete = 0;

                if skip_complete > 0 {
                    skip_complete -= 1;
                    continue;
                }

                for lidar_packet in complete_buf.iter() {
                    for block in lidar_packet.iter() {
                        let encoder_angle = 2.
                            * PI
                            * (1.
                                - (block.channels_header.measurement_id as f32
                                    / SCAN_WIDTH as f32));
                        let x_trans = offset_x * encoder_angle.cos();
                        let y_trans = offset_x * encoder_angle.sin();
                        for (row, (point, (azimuth, roh))) in block
                            .channels
                            .iter()
                            .zip(azimuth_roh_lut.iter())
                            .enumerate()
                        {
                            let dist = point.info_ret1.get_distance() - nvec;
                            let total_angle = encoder_angle + azimuth;
                            let x = (dist * total_angle.cos() * roh.cos() + x_trans)
                                .min(10_000.)
                                .max(-10000.);
                            let y = (dist * total_angle.sin() * roh.cos() + y_trans)
                                .min(10_000.)
                                .max(-10000.);
                            let z = (dist * roh.sin() + offset_z).min(10_000.).max(-10000.);
                            writer.push(&PcdPoint { x, y, z })?;
                            const FACTOR: f32 = 0.01;
                            const OFFSET: f32 = 0.;

                            let val = (dist * FACTOR + OFFSET).min(255.).max(0.) as u8;
                            min = min.min(val as f32);
                            max = max.max(val as f32);
                            let col = (total_angle / (PI * 2.) * (SCAN_WIDTH) as f32) as usize
                                % SCAN_WIDTH as usize;
                            image[row * 1024 + col] = val;
                        }
                    }
                }
                break;
            }

            // let points = pcd_converter.convert(lidar_packet)?;
            // println!("Output {}", points.len());
            // assert!(points.len() as u16 == pcd_converter.columns_per_revolution());
            // cnt += 1;

            // if cnt == 1000 {
            //     break;
            // }
        }
        let file = std::fs::File::create("out.png")?;
        let e = image::png::PngEncoder::new(file);
        e.write_image(&image, SCAN_WIDTH as _, LAYERS as _, image::ColorType::L8)?;

        writer.finish()?;
        std::fs::write(target, buf)?;

        println!("Min {min}, Max {max}");
        println!("Cnt {cnt}");
        panic!();
        Ok(())
    }

    #[derive(PcdSerialize)]
    pub struct PcdPoint {
        x: f32,
        y: f32,
        z: f32,
    }
}
