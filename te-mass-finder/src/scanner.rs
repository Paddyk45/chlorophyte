use matscan_tcp::StatelessTcpWriteHalf;

/// The thread that spews SYN packets
pub fn synner(
    ips: impl Iterator,
    write: StatelessTcpWriteHalf,
) {
    todo!();
}