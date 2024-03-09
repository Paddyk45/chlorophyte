#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod model;
mod scanner;

use crate::scanner::{garbage_collector, get_found_servers};
use chlorophyte_terraria_protocol::packet::{C2SConnect, WriteTerrariaPacket};
use log::{info, Level};
use matscan_ranges::exclude;
use matscan_ranges::targets::{ScanRange, ScanRanges};
use matscan_tcp::{SourcePort, StatelessTcp};
use std::env::{args, var};
use std::io::{stdout, Write};
use std::process::exit;
use std::thread::{sleep, spawn};
use std::time::Duration;

fn main() {
    ctrlc::set_handler(|| {
        dbg!(get_found_servers());
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
    let splashes = include_str!("../../assets/splashes.txt").lines().collect::<Vec<&str>>();
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
    let tcp_w = tcp.write.clone();
    let synner = spawn(|| scanner::synner(ranges, tcp_w));
    spawn(|| scanner::receiver(tcp.write, tcp.read));
    spawn(|| garbage_collector());
    synner.join().unwrap();
    println!("Sleeping 3 seconds...");
    sleep(Duration::from_secs(3));

    dbg!(get_found_servers());
}
