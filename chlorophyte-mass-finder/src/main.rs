#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod model;
mod scanner;

use chrono::Local;
use log::{info, Level};
use matscan_ranges::exclude;
use matscan_ranges::targets::{ScanRange, ScanRanges};
use matscan_tcp::{SourcePort, StatelessTcp};
use std::env::{args, var};
use std::fs::File;
use std::io::Write;
use std::net::SocketAddrV4;
use std::process::exit;
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

fn main() {
    ctrlc::set_handler(|| {
        dbg!(scanner::get_found_servers());
        exit(130);
    })
    .unwrap();
    if var("RUST_LOG").is_err() {
        simple_logger::init_with_level(Level::Info).unwrap();
    } else {
        simple_logger::init_with_env().unwrap();
    }
    let basic = figlet_rs::FIGfont::from_content(include_str!("../../assets/basic.flf")).unwrap();
    let mut banner = basic.convert("CHLOROPHYTE").unwrap();
    banner.height = 6;
    let splashes = include_str!("../../assets/splashes.txt")
        .lines()
        .collect::<Vec<&str>>();
    let splash = splashes[fastrand::usize(..splashes.len())];
    println!("{banner}{splash}\n");
    eprintln!("Chlorophyte MassFinder - https://github.com/Paddyk45/chlorophyte");
    let mut ranges = ScanRanges::new();

    let Some(includes) = args().nth(1) else {
        panic!("No range specified");
    };
    let includes = includes
        .split(',')
        .map(|i| i.parse::<ScanRange>().expect("Failed to parse scan range"))
        .collect::<Vec<ScanRange>>();
    ranges.extend(includes);

    let before_exclude = ranges.count();
    ranges.exclude_ranges(exclude::parse(include_str!("exclude.conf")).unwrap());
    info!("Excluded {} IP-addresses", before_exclude - ranges.count());

    let tcp = StatelessTcp::new(SourcePort::Range {
        min: 61000,
        max: 65000,
    });
    info!(
        "Scanning {} ranges @ {} IP-addresses",
        ranges.ranges().len(),
        ranges.count()
    );
    let start_time = Instant::now();
    let mut tcp_w = tcp.write.clone();
    spawn(|| scanner::receiver(tcp_w, tcp.read));
    spawn(|| scanner::garbage_collector());
    tcp_w = tcp.write.clone();
    scanner::synner(ranges, tcp_w);
    println!("SYNner done! Sleeping 3 seconds...");
    sleep(Duration::from_secs(3));

    let mut found_servers = scanner::get_found_servers();
    println!("Found {} Terraria servers!", found_servers.len());

    if start_time.elapsed().as_secs() > 60 * 60 * 2 {
        println!("Scan start was more than 2 hours ago, rescanning...");
        scanner::clear();
        let rescan_ranges = found_servers
            .iter()
            .map(|s| s.address)
            .collect::<Vec<SocketAddrV4>>()
            .into();
        scanner::synner(rescan_ranges, tcp.write);
        found_servers = scanner::get_found_servers();
    }
    let file_name = format!(
        "chlorophyte_mass_finder_results-{}.txt",
        Local::now().format("%y-%m-%d_%H_%M_%S")
    );
    let mut f = File::create(&file_name).expect("Failed to open files");
    for s in &found_servers {
        f.write_all(format!("{} {:?}\n", s.address, s.connection_request_result).as_bytes())
            .expect("Failed to write line to file");
    }

    println!("Results written to {file_name}");
}
