use image::{ImageEncoder, Luma};
use imageproc::contours::Contour;
use pcap::Capture;
use pcd_rs::{DataKind, PcdSerialize, WriterInit};
use std::{f32::consts::PI, io::Cursor, path::PathBuf};

use ouster_rs_ce::{
    Aggregator, CartesianIterator, DualMode, Mode, OusterConfig, OusterPacket, SingleMode,
};

const UDP_HEADER_SIZE: usize = 42;

#[test]
fn ouster_pcd_64() -> Result<(), Box<dyn std::error::Error>> {
    ouster_pcd_converter::<64, DualMode<16, 128>>(
        "OS-0-64-U02_v3.0.1_1024x10_20230510_135903.json",
        "OS-0-64-U02_v3.0.1_1024x10_20230510_135903-000.pcap",
    )
}

#[test]
fn ouster_pcd_128() -> Result<(), Box<dyn std::error::Error>> {
    ouster_pcd_converter::<128, DualMode<16, 128>>(
        "OS-0-128_v3.0.1_1024x10_20230510_134250.json",
        "OS-0-128_v3.0.1_1024x10_20230510_134250-000.pcap",
    )
}

#[test]
fn ouster_pcd_2047() -> Result<(), Box<dyn std::error::Error>> {
    ouster_pcd_converter::<128, DualMode<16, 128>>(
        "2023122_2047_OS-0-128_122313000118.json",
        "2023122_2047_OS-0-128_122313000118.pcap",
    )
}
#[test]
fn ouster_pcd_128rows_18_feb() -> Result<(), Box<dyn std::error::Error>> {
    ouster_pcd_converter::<128, DualMode<16, 128>>(
        "20240218_1622_OS-0-128_122403000369.json",
        "20240218_1622_OS-0-128_122403000369.pcap",
    )
}
#[test]
fn ouster_pcd_128_single() -> Result<(), Box<dyn std::error::Error>> {
    ouster_pcd_converter::<128, SingleMode<16, 128>>(
        "single_20240218_1625_OS-0-128_122403000369.json",
        "single_20240218_1625_OS-0-128_122403000369.pcap",
    )
}
fn ouster_pcd_converter<const LAYERS: usize, TMode: Mode>(
    test_json_path: &str,
    test_pcap_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load pcap file
    let home = PathBuf::from(env!("HOME"));
    let test_files = home.join("Downloads");
    let target = test_files.join("demo.pcd");

    let data = std::fs::read(test_files.join(test_json_path))?;
    let config: OusterConfig = serde_json::from_slice(&data)?;
    let mut cap = Capture::from_file(test_files.join(test_pcap_file))?;
    cap.filter("udp", true)?;

    let mut min = f32::MAX;
    let mut max = f32::MIN;

    let mut redefinitions = 0;
    let mut skip_complete = 10;
    let scan_width: u16 = config.lidar_data_format.columns_per_frame;

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

    let mut image = vec![0u8; scan_width as usize * LAYERS];
    let mut aggregator = Aggregator::new(scan_width as usize);

    while let Ok(packet) = cap.next_packet() {
        let slice = &packet.data[UDP_HEADER_SIZE..];
        if slice.len() != std::mem::size_of::<OusterPacket<16, LAYERS, TMode>>() {
            continue;
        }
        let lidar_packet = OusterPacket::<16, LAYERS, TMode>::from_maybe_unaligned(slice)?;
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

                //const FACTOR: f32 = 0.03;
                //const OFFSET: f32 = -80.;
                const FACTOR: f32 = 255. / 0.000001;
                const OFFSET: f32 = 0.;
                let val = (distance as f32 * FACTOR + OFFSET).min(255.).max(0.) as u8;
                min = min.min(val as f32);
                max = max.max(val as f32);
                let col = ((polar_point.azimuth / (PI * 2.) * scan_width as f32)
                    + scan_width as f32) as usize
                    % scan_width as usize;
                let image_idx = (idx % LAYERS) * (scan_width as usize) + col;
                image[image_idx] = val;
            }
            let mut dist = complete_buf.iter_points_flat(&config).collect::<Vec<_>>();
            dist.sort();
            println!(
                "\n50%: {}, 90%: {}",
                dist[dist.len() / 2],
                dist[dist.len() / 10 * 9]
            );

            break;
        }
    }
    println!("\n Redefinitions: {redefinitions}");
    let image = image::GrayImage::from_vec(scan_width as _, LAYERS as _, image).unwrap();
    let median = imageproc::filter::median_filter(&image, 2, 2);
    //let median = imageproc::filter::sharpen3x3(&median);
    let contours = imageproc::contours::find_contours::<i32>(&median); //Vec::<Contour<i32>>::new(); //
    for contour in contours {
        for p in contour.points {
            //sharp[(p.x as _, p.y as _)] = Luma([255]);
        }
    }
    image
        .save_with_format("out.png", image::ImageFormat::Png)
        .unwrap();
    median
        .save_with_format("median.png", image::ImageFormat::Png)
        .unwrap();

    pcd_writer.finish()?;
    println!("Write PCD to {:?}", target);
    std::fs::write(target, buf)?;

    println!("Min {min}, Max {max}");

    println!("Statistics {:?}", aggregator.get_statistics());
    Ok(())
}

#[derive(PcdSerialize)]
pub struct PcdPoint {
    x: f32,
    y: f32,
    z: f32,
}
