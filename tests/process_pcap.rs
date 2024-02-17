use image::ImageEncoder;
use pcap::Capture;
use pcd_rs::{DataKind, PcdSerialize, WriterInit};
use std::{f32::consts::PI, io::Cursor, path::PathBuf};

use ouster_rs_ce::{Aggregator, CartesianIterator, OusterConfig, OusterPacket};

const UDP_HEADER_SIZE: usize = 42;

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
    let config: OusterConfig = serde_json::from_slice(&data)?;
    let mut cap = Capture::from_file(test_files.join(test_pcap_file))?;
    cap.filter("udp", true)?;

    let mut min = f32::MAX;
    let mut max = f32::MIN;
    let mut skip_complete = 130;
    const SCAN_WIDTH: u16 = 1024;

    //const CAPTURE_POINTS: usize = 70974464;
    const CAPTURE_POINTS: usize = 151072;
    let mut buf = Vec::new();
    let mut pcd_writer = WriterInit {
        width: CAPTURE_POINTS as _,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: None,
    }
    .build_from_writer(Cursor::new(&mut buf))?;

    let mut image = vec![0u8; 1024 * LAYERS];
    let mut aggregator = Aggregator::default();

    while let Ok(packet) = cap.next_packet() {
        let slice = &packet.data[UDP_HEADER_SIZE..];
        if slice.len() != std::mem::size_of::<OusterPacket<16, LAYERS>>() {
            continue;
        }
        let lidar_packet = OusterPacket::<16, LAYERS>::from_maybe_unaligned(slice)?;
        if let Some(complete_buf) = aggregator.put_data_value(lidar_packet.clone()) {
            if skip_complete > 0 {
                skip_complete -= 1;
                continue;
            }

            let iter = CartesianIterator::from_config(&config);
            for (idx, (distance, polar_point)) in
                complete_buf.iter_points_flat(&config).zip(iter).enumerate()
            {
                let (x, y, z) = polar_point.calc_xyz(distance as f32);

                let x = x.min(20_000.).max(-20000.);
                let y = y.min(20_000.).max(-20000.);
                let z = z.min(20_000.).max(-20000.);
                pcd_writer.push(&PcdPoint { x, y, z })?;

                const FACTOR: f32 = 0.01;
                const OFFSET: f32 = 0.;
                let val = (distance as f32 * FACTOR + OFFSET).min(255.).max(0.) as u8;
                min = min.min(val as f32);
                max = max.max(val as f32);
                let col = (polar_point.azimuth / (PI * 2.) * (SCAN_WIDTH) as f32) as usize
                    % SCAN_WIDTH as usize;
                image[(idx % LAYERS) * 1024 + col] = val;
            }

            break;
        }
    }
    let file = std::fs::File::create("out.png")?;
    image::png::PngEncoder::new(file).write_image(
        &image,
        SCAN_WIDTH as _,
        LAYERS as _,
        image::ColorType::L8,
    )?;

    pcd_writer.finish()?;
    std::fs::write(target, buf)?;

    println!("Min {min}, Max {max}");
    Ok(())
}

#[derive(PcdSerialize)]
pub struct PcdPoint {
    x: f32,
    y: f32,
    z: f32,
}
