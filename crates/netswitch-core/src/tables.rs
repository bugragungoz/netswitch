//! Reads the tables `GetExtendedTcpTable` and `GetExtendedUdpTable` fill (the `*_OWNER_PID`
//! classes). Their layout is fixed: a 32-bit row count, then the rows. Reading them from bytes
//! here, instead of through the API's structs, keeps the parsing testable on any system.
//!
//! In every row, addresses are in network byte order and ports sit in network byte order in the
//! low 16 bits of a 32-bit field.

use crate::model::{Protocol, Socket, TcpState};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    V4,
    V6,
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    bytes.get(at..at + 4).map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
}

fn port(field: u32) -> u16 {
    u16::from_be((field & 0xffff) as u16)
}

fn v4(field: u32) -> IpAddr {
    IpAddr::V4(Ipv4Addr::from(field.to_ne_bytes()))
}

fn v6(bytes: &[u8], at: usize) -> Option<IpAddr> {
    let raw: [u8; 16] = bytes.get(at..at + 16)?.try_into().ok()?;
    Some(IpAddr::V6(Ipv6Addr::from(raw)))
}

/// Rows of a table, each `row_len` bytes after the 4-byte count. A count larger than the buffer
/// holds is cut to what is there.
fn rows(bytes: &[u8], row_len: usize) -> impl Iterator<Item = &[u8]> {
    let count = u32_at(bytes, 0).unwrap_or(0) as usize;
    let available = bytes.len().saturating_sub(4) / row_len;
    (0..count.min(available)).map(move |i| &bytes[4 + i * row_len..4 + (i + 1) * row_len])
}

/// MIB_TCPTABLE_OWNER_PID or MIB_TCP6TABLE_OWNER_PID.
pub fn parse_tcp(bytes: &[u8], family: Family) -> Vec<Socket> {
    match family {
        Family::V4 => rows(bytes, 24)
            .filter_map(|r| {
                let state = u32_at(r, 0)?;
                Some(Socket {
                    pid: u32_at(r, 20)?,
                    protocol: Protocol::Tcp,
                    local: SocketAddr::new(v4(u32_at(r, 4)?), port(u32_at(r, 8)?)),
                    remote: Some(SocketAddr::new(v4(u32_at(r, 12)?), port(u32_at(r, 16)?))),
                    state: TcpState::from_mib(state),
                })
            })
            .collect(),
        Family::V6 => rows(bytes, 56)
            .filter_map(|r| {
                Some(Socket {
                    pid: u32_at(r, 52)?,
                    protocol: Protocol::Tcp,
                    local: SocketAddr::new(v6(r, 0)?, port(u32_at(r, 20)?)),
                    remote: Some(SocketAddr::new(v6(r, 24)?, port(u32_at(r, 44)?))),
                    state: TcpState::from_mib(u32_at(r, 48)?),
                })
            })
            .collect(),
    }
}

/// MIB_UDPTABLE_OWNER_PID or MIB_UDP6TABLE_OWNER_PID.
pub fn parse_udp(bytes: &[u8], family: Family) -> Vec<Socket> {
    match family {
        Family::V4 => rows(bytes, 12)
            .filter_map(|r| {
                Some(Socket {
                    pid: u32_at(r, 8)?,
                    protocol: Protocol::Udp,
                    local: SocketAddr::new(v4(u32_at(r, 0)?), port(u32_at(r, 4)?)),
                    remote: None,
                    state: None,
                })
            })
            .collect(),
        Family::V6 => rows(bytes, 28)
            .filter_map(|r| {
                Some(Socket {
                    pid: u32_at(r, 24)?,
                    protocol: Protocol::Udp,
                    local: SocketAddr::new(v6(r, 0)?, port(u32_at(r, 20)?)),
                    remote: None,
                    state: None,
                })
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ne(v: u32) -> [u8; 4] {
        v.to_ne_bytes()
    }

    fn port_field(p: u16) -> u32 {
        // The API stores the port's network-order bytes in the low half of the field.
        u16::from_ne_bytes(p.to_be_bytes()) as u32
    }

    fn addr_field(a: [u8; 4]) -> u32 {
        u32::from_ne_bytes(a)
    }

    #[test]
    fn reads_ipv4_tcp_rows() {
        let mut b = Vec::new();
        b.extend(ne(2));
        for (state, local, lport, remote, rport, pid) in [
            (5u32, [192, 168, 1, 10], 50123u16, [142, 250, 1, 1], 443u16, 1234u32),
            (2, [0, 0, 0, 0], 8080, [0, 0, 0, 0], 0, 4),
        ] {
            b.extend(ne(state));
            b.extend(ne(addr_field(local)));
            b.extend(ne(port_field(lport)));
            b.extend(ne(addr_field(remote)));
            b.extend(ne(port_field(rport)));
            b.extend(ne(pid));
        }
        let rows = parse_tcp(&b, Family::V4);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].local, "192.168.1.10:50123".parse().unwrap());
        assert_eq!(rows[0].remote, Some("142.250.1.1:443".parse().unwrap()));
        assert_eq!(rows[0].state, Some(TcpState::Established));
        assert_eq!(rows[0].pid, 1234);
        assert_eq!(rows[1].state, Some(TcpState::Listen));
        assert_eq!(rows[1].local.port(), 8080);
    }

    #[test]
    fn reads_ipv6_tcp_rows() {
        let local: Ipv6Addr = "2001:db8::10".parse().unwrap();
        let remote: Ipv6Addr = "2a00:1450::200e".parse().unwrap();
        let mut b = Vec::new();
        b.extend(ne(1));
        b.extend(local.octets());
        b.extend(ne(0));
        b.extend(ne(port_field(51000)));
        b.extend(remote.octets());
        b.extend(ne(0));
        b.extend(ne(port_field(443)));
        b.extend(ne(5));
        b.extend(ne(99));
        let rows = parse_tcp(&b, Family::V6);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].local, SocketAddr::new(IpAddr::V6(local), 51000));
        assert_eq!(rows[0].remote, Some(SocketAddr::new(IpAddr::V6(remote), 443)));
        assert_eq!(rows[0].pid, 99);
    }

    #[test]
    fn reads_udp_rows() {
        let mut b = Vec::new();
        b.extend(ne(1));
        b.extend(ne(addr_field([0, 0, 0, 0])));
        b.extend(ne(port_field(5353)));
        b.extend(ne(2000));
        let rows = parse_udp(&b, Family::V4);
        assert_eq!(
            rows,
            vec![Socket {
                pid: 2000,
                protocol: Protocol::Udp,
                local: "0.0.0.0:5353".parse().unwrap(),
                remote: None,
                state: None
            }]
        );

        let mut b6 = Vec::new();
        b6.extend(ne(1));
        b6.extend(Ipv6Addr::UNSPECIFIED.octets());
        b6.extend(ne(0));
        b6.extend(ne(port_field(5355)));
        b6.extend(ne(3000));
        let rows = parse_udp(&b6, Family::V6);
        assert_eq!(rows[0].local.port(), 5355);
        assert_eq!(rows[0].pid, 3000);
    }

    #[test]
    fn a_short_buffer_gives_only_whole_rows() {
        let mut b = Vec::new();
        b.extend(ne(5)); // claims five rows
        b.extend([0u8; 24 + 10]); // holds one and a part
        assert_eq!(parse_tcp(&b, Family::V4).len(), 1);
        assert!(parse_tcp(&[], Family::V4).is_empty());
    }
}
