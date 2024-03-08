use std::net::SocketAddrV4;
use std::time::Instant;
#[derive(Clone, Debug)]
pub enum ConnectionRequestResult {
    Approved,
    PasswordRequired,
    Booted(/* Reason: */ String),
}

#[derive(Clone, Debug)]
pub struct TerrariaServer {
    pub address: SocketAddrV4,
    /// How the server reacted to the connection request
    pub connection_request_result: ConnectionRequestResult,
}

#[derive(Clone, Debug)]
pub struct ConnectionState {
    /// When we sent the SYN - used for garbage collection
    pub syn_time: Instant,
    /// Whether the TCP handshake is complete (SYN, SYN+ACK, ACK)
    /// and we sent a connection request packet
    pub handshake_done: bool,
    /// Whether the connection was finished or reset
    pub closed: bool,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            handshake_done: false,
            syn_time: Instant::now(),
            closed: false,
        }
    }
}
