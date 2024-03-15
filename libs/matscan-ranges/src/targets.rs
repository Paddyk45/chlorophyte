use crate::{Ipv4Range, Ipv4Ranges};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRange {
    pub addr_start: Ipv4Addr,
    pub addr_end: Ipv4Addr,
    pub port_start: u16,
    pub port_end: u16,
}

impl ScanRange {
    pub fn count_addresses(&self) -> usize {
        (u32::from(self.addr_end) as u64 - u32::from(self.addr_start) as u64 + 1) as usize
    }

    pub fn count_ports(&self) -> usize {
        ((self.port_end - self.port_start) + 1) as usize
    }

    /// Count the number of combinations of addresses and ports in this range.
    pub fn count(&self) -> usize {
        self.count_addresses() * self.count_ports()
    }

    /// Get the address and port at the given index.
    pub fn index(&self, index: usize) -> SocketAddrV4 {
        let port_count = self.count_ports();
        let addr_index = index / port_count;
        let port_index = index % port_count;
        let addr = u32::from(self.addr_start) + addr_index as u32;
        let port = self.port_start + port_index as u16;
        SocketAddrV4::new(Ipv4Addr::from(addr), port)
    }

    pub fn single(addr: Ipv4Addr, port: u16) -> Self {
        Self {
            addr_start: addr,
            addr_end: addr,
            port_start: port,
            port_end: port,
        }
    }
    pub fn single_port(addr_start: Ipv4Addr, addr_end: Ipv4Addr, port: u16) -> Self {
        Self {
            addr_start,
            addr_end,
            port_start: port,
            port_end: port,
        }
    }
    pub fn single_address(addr: Ipv4Addr, port_start: u16, port_end: u16) -> Self {
        Self {
            addr_start: addr,
            addr_end: addr,
            port_start,
            port_end,
        }
    }
    pub fn multi_address_port(
        addr_start: Ipv4Addr,
        addr_end: Ipv4Addr,
        port_start: u16,
        port_end: u16,
    ) -> Self {
        Self {
            addr_start,
            addr_end,
            port_start,
            port_end,
        }
    }
}

impl FromStr for ScanRange {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((addr, port)) = s.split_once(':') else {
            return Err("No port specified");
        };

        // parse the ip address range
        let is_slash = addr.contains('/');
        let is_hyphen = addr.contains('-');

        if is_slash && is_hyphen {
            return Err("Invalid range - contains both - and /");
        }

        let range = if is_slash {
            let mut parts = addr.split('/');

            let ip = parts.next().unwrap();
            let mask = parts.next().unwrap();

            let mask = 32
                - mask
                    .parse::<u8>()
                    .map_err(|_| "Failed to parse subnet mask as u8")?;

            let mask_bits = 2u32.pow(mask as u32) - 1;

            let ip_net_addr =
                Ipv4Addr::from_str(ip).map_err(|_| "Failed to parse IP net address as Ipv4Addr")?;
            let ip_u32 = u32::from(ip_net_addr);

            let addr_start = Ipv4Addr::from(ip_u32 & !mask_bits);
            let addr_end = Ipv4Addr::from(ip_u32 | mask_bits);

            Ipv4Range {
                start: addr_start,
                end: addr_end,
            }
        } else if is_hyphen {
            let mut parts = addr.split('-');

            let ip_start = parts.next().unwrap();
            let ip_end = parts.next().unwrap();

            let ip_start = Ipv4Addr::from_str(ip_start)
                .map_err(|_| "Could not parse address start as Ipv4Addr")?;
            let ip_end = Ipv4Addr::from_str(ip_end)
                .map_err(|_| "Could not parse address end as Ipv4Addr")?;
            if ip_start > ip_end {
                return Err("Start address is bigger than the end address");
            }

            Ipv4Range {
                start: ip_start,
                end: ip_end,
            }
        } else {
            let addr =
                Ipv4Addr::from_str(addr).map_err(|_| "Could not parse single address as Ipv4Addr")?;
            Ipv4Range::single(addr)
        };

        // parse the port range
        let is_hyphen = port.contains("-");
        let (port_start, port_end) = if is_hyphen {
            let (s, e) = port.split_once("-").unwrap();
            let s = s
                .parse::<u16>()
                .map_err(|_| "Could not parse port start as u16")?;
            let e = e
                .parse::<u16>()
                .map_err(|_| "Could not parse port end as u16")?;
            (s, e)
        } else {
            let port = port
                .parse::<u16>()
                .map_err(|_| "Failed to parse port as u16")?;
            (port, port)
        };

        Ok(Self {
            addr_start: range.start,
            addr_end: range.end,
            port_start,
            port_end,
        })
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct ScanRanges {
    /// The ranges in order of `addr_start`.
    ranges: Vec<ScanRange>,
}

impl ScanRanges {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add to the set of ranges. There is no push function because it'd be too
    /// inefficient, you can call this with a single-item vec if you really need
    /// to.
    pub fn extend(&mut self, ranges: Vec<ScanRange>) {
        self.ranges.extend(ranges);
        self.ranges.sort_by_key(|r| r.addr_start.octets());
    }

    pub fn exclude_ranges(&mut self, ranges: Ipv4Ranges) {
        for range in ranges.ranges() {
            self.exclude(range);
        }
    }

    fn exclude(&mut self, exclude_range: &Ipv4Range) {
        let mut i = 0;
        while i < self.ranges.len() && self.ranges[i].addr_end < exclude_range.start {
            i += 1;
        }

        let mut queued_push = vec![];

        while i < self.ranges.len() && self.ranges[i].addr_start <= exclude_range.end {
            let range = &mut self.ranges[i];
            if range.addr_start >= exclude_range.start && range.addr_end <= exclude_range.end {
                // Range is fully contained in exclude.
                self.ranges.remove(i);
            } else if range.addr_start < exclude_range.start && range.addr_end > exclude_range.end {
                // Range fully contains the exclude, so split.
                let other_range = ScanRange {
                    addr_start: Ipv4Addr::from(u32::from(exclude_range.end) + 1),
                    addr_end: range.addr_end,
                    port_start: range.port_start,
                    port_end: range.port_end,
                };
                range.addr_end = Ipv4Addr::from(u32::from(exclude_range.start) - 1);
                queued_push.push(other_range);
                i += 1;
            } else if range.addr_start < exclude_range.start && range.addr_end <= exclude_range.end
            {
                // Cut off end.
                range.addr_end = Ipv4Addr::from(u32::from(exclude_range.start) - 1);
                i += 1;
            } else if range.addr_start >= exclude_range.start && range.addr_end > exclude_range.end
            {
                // Cut off start.

                // changing addr_start would change the position, so it's easier to just delete
                // it and add it later
                let range = self.ranges.remove(i);
                queued_push.push(ScanRange {
                    addr_start: Ipv4Addr::from(u32::from(exclude_range.end) + 1),
                    addr_end: range.addr_end,
                    port_start: range.port_start,
                    port_end: range.port_end,
                });
            } else {
                unreachable!();
            }
        }
        self.extend(queued_push);
        if self.is_empty() {
            println!("uh oh count is 0 after {:?}", exclude_range);
        }
    }

    /// Get the address and port at the given index.
    ///
    /// You should use [`Self::to_static`] and then call index on that.
    pub fn slow_index(&self, index: usize) -> SocketAddrV4 {
        let mut i = 0;
        let mut index = index;
        while i < self.ranges.len() {
            let range = &self.ranges[i];
            let count = range.count();
            if index < count {
                return range.index(index);
            }
            index -= count;
            i += 1;
        }
        panic!("index out of bounds");
    }

    /// Count the total number of targets that are going to be scanned.
    pub fn count(&self) -> usize {
        let mut total = 0;
        for range in &self.ranges {
            total += range.count();
        }
        total
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn ranges(&self) -> &Vec<ScanRange> {
        &self.ranges
    }

    pub fn to_static(self) -> StaticScanRanges {
        let mut ranges = Vec::with_capacity(self.ranges.len());
        let mut index = 0;
        for range in self.ranges {
            let count = range.count();
            ranges.push(StaticScanRange {
                count,
                range,
                index,
            });
            index += count;
        }
        StaticScanRanges {
            ranges,
            count: index,
        }
    }
}

impl From<Vec<SocketAddrV4>> for ScanRanges {
    fn from(addrs: Vec<SocketAddrV4>) -> Self {
        let mut ranges: Vec<ScanRange> = vec![];
        for socket_addr in addrs {
            ranges.push(ScanRange {
                addr_start: *socket_addr.ip(),
                addr_end: *socket_addr.ip(),
                port_start: socket_addr.port(),
                port_end: socket_addr.port(),
            })
        };
        let mut scan_ranges = ScanRanges::new();
        scan_ranges.extend(ranges);
        scan_ranges
    }
}

pub struct StaticScanRanges {
    pub ranges: Vec<StaticScanRange>,
    pub count: usize,
}

pub struct StaticScanRange {
    pub range: ScanRange,
    count: usize,
    index: usize,
}

impl StaticScanRanges {
    pub fn index(&self, index: usize) -> SocketAddrV4 {
        // binary search to find the range that contains the index
        let mut start = 0;
        let mut end = self.ranges.len();
        while start < end {
            let mid = (start + end) / 2;
            let range = &self.ranges[mid];
            if range.index + range.count <= index {
                start = mid + 1;
            } else if range.index > index {
                end = mid;
            } else {
                return range.range.index(index - range.index);
            }
        }
        panic!("index out of bounds");
    }
}
