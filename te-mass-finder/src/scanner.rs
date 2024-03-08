use crate::model::{ConnectionRequestResult, ConnectionState, TerrariaServer};
use ipnet::IpAdd;
use matscan_ranges::targets::ScanRanges;
use matscan_tcp::{StatelessTcpReadHalf, StatelessTcpWriteHalf};
use once_cell::sync::Lazy;
use pnet_packet::tcp::TcpFlags;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::SocketAddrV4;
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use log::info;
use te_terraria_protocol::packet::{
    C2SConnect, ReadTerrariaPacket, S2CConnectionApproved, S2CFatalError, S2CPasswordRequired,
    WriteTerrariaPacket,
};

static CONNECTIONS: Lazy<RwLock<HashMap<SocketAddrV4, ConnectionState>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static FOUND_SERVERS: Lazy<RwLock<Vec<TerrariaServer>>> = Lazy::new(|| RwLock::new(vec![]));

/// The thread that spews SYN packets
#[allow(clippy::needless_pass_by_value)]
pub fn synner(ranges: ScanRanges, mut tcp_w: StatelessTcpWriteHalf) {
    for range in ranges.ranges() {
        let mut addr = range.addr_start;
        let addr_end = range.addr_end;
        while addr <= addr_end {
            for port in range.port_start..=range.port_end {
                let addr = SocketAddrV4::new(addr, port);
                tcp_w.send_syn(addr, fastrand::u32(..u32::MAX - 100_000));
                CONNECTIONS
                    .write()
                    .unwrap()
                    .insert(addr, ConnectionState::default());
                sleep(Duration::from_millis(1) / 2);
            }
            addr = addr.saturating_add(1);
        }
    }
}

/// The thread that finishes the TCP handshake and handles incoming packets from the server
#[allow(clippy::significant_drop_tightening)]
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
            if FOUND_SERVERS.read().unwrap().iter().any(|s| s.address == addr) {
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

/// Removes connections that didn't send a SYN+ACK
pub fn garbage_collector() -> ! {
    loop {
        let conns = CONNECTIONS.read().unwrap().clone();
        let mut to_remove = vec![];
        conns
            .iter()
            .enumerate()
            .filter(|c| c.1 .1.syn_time.elapsed() > Duration::from_secs(7) || c.1 .1.closed)
            .for_each(|c| to_remove.push(c.1 .0));

        for i in to_remove {
            CONNECTIONS.write().unwrap().remove(i);
        }
    }
}

pub fn get_found_servers() -> Vec<TerrariaServer> {
    FOUND_SERVERS.read().unwrap().clone()
}
