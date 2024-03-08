#![warn(clippy::all, clippy::nursery, clippy::pedantic)]

mod model;
mod scanner;

use crate::scanner::{garbage_collector, get_found_servers};
use log::{info, Level};
use matscan_ranges::exclude;
use matscan_ranges::targets::{ScanRange, ScanRanges};
use matscan_tcp::{SourcePort, StatelessTcp};
use std::net::Ipv4Addr;
use std::thread::{sleep, spawn};
use std::time::Duration;
use te_terraria_protocol::packet::{C2SConnect, WriteTerrariaPacket};

fn main() {
    simple_logger::init_with_level(Level::Trace).unwrap();
    eprintln!("TerraEye - MassFinder v0.0.1 - https://github.com/Paddyk45/terra-eye");
    let mut conn_request_packet = vec![0u8; 0];
    conn_request_packet
        .write_terraria_packet(C2SConnect { version: 279 })
        .unwrap();
    // TODO: Add more range modes
    let mut ranges = ScanRanges::new();
    ranges.extend(vec![ScanRange::single_port(
        Ipv4Addr::new(50, 0, 0, 0),
        Ipv4Addr::new(52, 255, 255, 255),
        7777
    )]);
    let before_exclude = ranges.count();
    ranges.exclude_ranges(exclude::parse(include_str!("exclude.conf")).unwrap());
    info!("Excluded {} IP-addresses", before_exclude - ranges.count());

    let tcp = StatelessTcp::new(SourcePort::Range { min: 61000, max:65000 });
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
