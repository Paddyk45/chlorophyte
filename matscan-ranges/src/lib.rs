pub mod exclude;
pub mod targets;

use std::net::Ipv4Addr;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Ipv4Range {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
}

impl Ipv4Range {
    pub fn single(addr: Ipv4Addr) -> Self {
        Self {
            start: addr,
            end: addr,
        }
    }
}

pub struct Ipv4Ranges {
    ranges: Vec<Ipv4Range>,
}

impl Ipv4Ranges {
    pub fn new(mut ranges: Vec<Ipv4Range>) -> Self {
        ranges.sort_by_key(|r| r.start);
        Self { ranges }
    }

    pub fn contains(&self, addr: Ipv4Addr) -> bool {
        let mut start = 0;
        let mut end = self.ranges.len();
        while start < end {
            let mid = (start + end) / 2;
            let range = &self.ranges[mid];
            if range.end < addr {
                start = mid + 1;
            } else if range.start > addr {
                end = mid;
            } else {
                return true;
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn ranges(&self) -> &Vec<Ipv4Range> {
        &self.ranges
    }

    pub fn count(&self) -> usize {
        let mut total: u64 = 0;
        for range in &self.ranges {
            total += (u32::from(range.end) as u64) - (u32::from(range.start) as u64) + 1;
        }
        total as usize
    }
}
