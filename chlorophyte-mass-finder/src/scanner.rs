use crate::model::{ConnectionRequestResult, ConnectionState, TerrariaServer};
use chlorophyte_terraria_protocol::packet::{
    C2SConnect, ReadTerrariaPacket, S2CConnectionApproved, S2CFatalError, S2CPasswordRequired,
    WriteTerrariaPacket,
};
use ipnet::IpAdd;
use log::{info, trace};
use matscan_ranges::targets::ScanRanges;
use matscan_tcp::{StatelessTcpReadHalf, StatelessTcpWriteHalf, Throttler};
use once_cell::sync::Lazy;
use pnet_packet::tcp::TcpFlags;
use std::collections::HashMap;
use std::env::args;
use std::io::Cursor;
use std::net::SocketAddrV4;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::{Duration, Instant};

static CONNECTIONS: Lazy<RwLock<HashMap<SocketAddrV4, ConnectionState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static FOUND_SERVERS: Lazy<RwLock<Vec<TerrariaServer>>> = Lazy::new(|| RwLock::new(vec![]));

/// The thread that spews SYN packets
#[allow(clippy::needless_pass_by_value, clippy::cast_precision_loss)]
pub fn synner(ranges: ScanRanges, mut tcp_w: StatelessTcpWriteHalf) {
    let max_pps = args().nth(2).map_or(10_000, |pps| {
        pps.parse().expect("Failed to parse max pps as u64")
    });
    let addrs = ranges.count() as f64;
    let mut throttler = Throttler::new(max_pps);
    info!("Throttler is set to {max_pps} packets/s");

    let mut t = Instant::now();
    let mut p = 0usize;

    let mut batch_size = throttler.next_batch();
    let mut syns = 0f64;
    for range in ranges.ranges() {
        let mut addr = range.addr_start;
        let addr_end = range.addr_end;
        while addr <= addr_end {
            for port in range.port_start..=range.port_end {
                let addr = SocketAddrV4::new(addr, port);
                tcp_w.send_syn(addr, fastrand::u32(..u32::MAX - 100_000));
                p += 1;
                syns += 1.;
                if t.elapsed().as_nanos() >= Duration::from_secs(1).as_nanos() {
                    info!("Scanning @ ~{p} packets/s");
                    info!(
                        "{:.3}% done ({}/{} hosts done)",
                        (syns / addrs) * 100.,
                        syns,
                        addrs
                    );
                    t = Instant::now();
                    p = 0;
                }
                batch_size -= 1;
                if batch_size == 0 {
                    batch_size = throttler.next_batch();
                }
                CONNECTIONS
                    .write()
                    .unwrap()
                    .insert(addr, ConnectionState::default());
            }
            addr = addr.saturating_add(1);
        }
    }

    println!("SYNner done");
}

/// The thread that finishes the TCP handshake and handles incoming packets from the server
#[allow(clippy::significant_drop_tightening, clippy::too_many_lines)]
pub fn receiver(mut tcp_w: StatelessTcpWriteHalf, mut tcp_r: StatelessTcpReadHalf) -> ! {
    let mut conn_request_packet = vec![0u8; 0];
    conn_request_packet
        .write_terraria_packet(C2SConnect { version: 279 })
        .unwrap();
    loop {
        let Some((ipv4, tcp)) = tcp_r.recv() else {
            sleep(Duration::from_millis(2));
            continue;
        };
        let addr = SocketAddrV4::new(ipv4.source, tcp.source);
        let mut wguard = CONNECTIONS.write().unwrap();
        let Some(conn) = wguard.get_mut(&addr) else {
            tcp_w.send_rst(addr, tcp.destination, tcp.acknowledgement, tcp.sequence);
            continue;
        };

        if conn.closed {
            continue;
        }

        // SYN+ACK
        if tcp.flags & TcpFlags::SYN != 0 && tcp.flags & TcpFlags::ACK != 0 {
            tcp_w.send_ack(addr, tcp.destination, tcp.acknowledgement, tcp.sequence + 1);
            tcp_w.send_data(
                addr,
                tcp.destination,
                tcp.acknowledgement,
                tcp.sequence + 1,
                &conn_request_packet,
            );
            conn.handshake_done = true;
        }

        // PSH - Terraria packet
        if tcp.flags & TcpFlags::PSH != 0 {
            tcp_w.send_ack(
                addr,
                tcp.destination,
                tcp.acknowledgement,
                tcp.sequence + u32::try_from(tcp.payload.len()).unwrap(),
            );
            let Some(packet_id) = tcp.payload.get(2).copied() else {
                tcp_w.send_rst(addr, tcp.destination, tcp.acknowledgement, tcp.sequence);
                continue;
            };
            if FOUND_SERVERS
                .read()
                .unwrap()
                .iter()
                .any(|s| s.address == addr)
            {
                continue;
            }
            match packet_id {
                2 => {
                    if let Ok(packet) =
                        Cursor::new(&tcp.payload).read_terraria_packet::<S2CFatalError>()
                    {
                        info!("Found server, but I got booted: {addr}");
                        FOUND_SERVERS.write().unwrap().push(TerrariaServer {
                            address: addr,
                            connection_request_result: ConnectionRequestResult::Booted(
                                packet.error,
                            ),
                        });
                    }
                }
                3 if Cursor::new(&tcp.payload)
                    .read_terraria_packet::<S2CConnectionApproved>()
                    .is_ok() =>
                {
                    info!("Found server: {addr}");
                    FOUND_SERVERS.write().unwrap().push(TerrariaServer {
                        address: addr,
                        connection_request_result: ConnectionRequestResult::Approved,
                    });
                }
                9 => {
                    info!("Found server: {addr}");
                    FOUND_SERVERS.write().unwrap().push(TerrariaServer {
                        address: addr,
                        connection_request_result: ConnectionRequestResult::Approved,
                    });
                }
                37 if Cursor::new(&tcp.payload)
                    .read_terraria_packet::<S2CPasswordRequired>()
                    .is_ok() =>
                {
                    info!("Found password-protected server: {addr}");
                    FOUND_SERVERS.write().unwrap().push(TerrariaServer {
                        address: addr,
                        connection_request_result: ConnectionRequestResult::PasswordRequired,
                    });
                }
                82 => continue,
                _ => {}
            }
            tcp_w.send_rst(addr, tcp.destination, tcp.acknowledgement, tcp.sequence);
        }

        // RST
        if tcp.flags & TcpFlags::RST != 0 {
            conn.closed = true;
        }

        // FIN
        if tcp.flags & TcpFlags::FIN != 0 {
            tcp_w.send_fin(addr, tcp.destination, tcp.acknowledgement, tcp.sequence);
            conn.closed = true;
        }
    }
}

/// Removes connections that didn't send a SYN+ACK or were reset
pub fn garbage_collector() -> ! {
    let timeout = Duration::from_secs(7);
    loop {
        let conns = CONNECTIONS.read().unwrap().clone();
        let mut to_remove = vec![];
        conns
            .iter()
            .enumerate()
            .filter(|c| c.1 .1.syn_time.elapsed() > timeout || c.1.1.closed)
            .for_each(|c| to_remove.push(c.1.0));
        if !to_remove.is_empty() {
            trace!("[gc] removing {} connections", to_remove.len());
        }
        for i in to_remove {
            CONNECTIONS.write().unwrap().remove(i);
        }
        sleep(Duration::from_millis(40));
    }
}

pub fn get_found_servers() -> Vec<TerrariaServer> {
    FOUND_SERVERS.read().unwrap().clone()
}
