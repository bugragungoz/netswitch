//! The Event Tracing for Windows events netSwitch reads, and the parts of them it understands.
//! Kept apart from the trace itself so the rules are testable on any system.

use crate::model::{Direction, Protocol};
use crate::net::canonical;
use std::net::IpAddr;

/// Microsoft-Windows-Kernel-Network: one event per send or receive, with the process, the size and
/// both addresses.
pub const KERNEL_NETWORK: &str = "7dd42a49-5329-4832-8dfd-43d979153a88";
/// The provider's IPv4 and IPv6 keywords.
pub const KERNEL_NETWORK_KEYWORDS: u64 = 0x10 | 0x20;

/// Microsoft-Windows-DNS-Client: names programs asked for and the addresses they got.
pub const DNS_CLIENT: &str = "1c95126e-7eea-49a9-a3fe-a378b03ddb4d";
/// "DNS query is completed": QueryName, QueryType, QueryStatus, QueryResults.
pub const DNS_QUERY_COMPLETED: u16 = 3008;

/// What a Kernel-Network event carries, by its id. Connect, disconnect, retransmit and copy events
/// carry no new bytes and are left out, so nothing is counted twice.
pub fn kernel_network_transfer(event_id: u16) -> Option<(Protocol, Direction)> {
    match event_id {
        10 | 26 => Some((Protocol::Tcp, Direction::Out)),
        11 | 27 => Some((Protocol::Tcp, Direction::In)),
        42 | 58 => Some((Protocol::Udp, Direction::Out)),
        43 | 59 => Some((Protocol::Udp, Direction::In)),
        _ => None,
    }
}

/// The event's port fields hold the port in network byte order.
pub fn event_port(raw: u16) -> u16 {
    u16::from_be(raw)
}

/// The addresses in the QueryResults of a completed DNS query, such as
/// `type:  5 www.example.com;::ffff:93.184.216.34;2606:2800:220:1::248;`. Aliases (CNAME, "type: 5")
/// are skipped; IPv4 addresses written as IPv6 come back as IPv4.
pub fn dns_result_addresses(results: &str) -> Vec<IpAddr> {
    results
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty() && !part.starts_with("type:"))
        .filter_map(|part| part.parse::<IpAddr>().ok())
        .map(canonical)
        .collect()
}

/// The name as people write it: lowercase, without the root dot.
pub fn dns_name(query_name: &str) -> Option<String> {
    let name = query_name.trim().trim_end_matches('.').to_lowercase();
    (!name.is_empty() && name.contains('.') && !name.ends_with(".arpa")).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfers_by_event_id() {
        assert_eq!(kernel_network_transfer(10), Some((Protocol::Tcp, Direction::Out)));
        assert_eq!(kernel_network_transfer(27), Some((Protocol::Tcp, Direction::In)));
        assert_eq!(kernel_network_transfer(42), Some((Protocol::Udp, Direction::Out)));
        assert_eq!(kernel_network_transfer(59), Some((Protocol::Udp, Direction::In)));
        for other in [12, 13, 14, 15, 18, 28, 34] {
            assert_eq!(
                kernel_network_transfer(other),
                None,
                "event {other} carries no new bytes"
            );
        }
    }

    #[test]
    fn ports_are_swapped_from_network_order() {
        assert_eq!(event_port(u16::from_ne_bytes(443u16.to_be_bytes())), 443);
    }

    #[test]
    fn dns_results_give_the_addresses() {
        let found = dns_result_addresses("type:  5 www.example.com.cdn.net;::ffff:93.184.216.34;2606:2800:220:1::248;");
        assert_eq!(
            found,
            vec![
                "93.184.216.34".parse::<IpAddr>().unwrap(),
                "2606:2800:220:1::248".parse().unwrap()
            ]
        );
        assert!(dns_result_addresses("").is_empty());
        assert!(dns_result_addresses("type:  5 a.b.c;").is_empty());
    }

    #[test]
    fn dns_names_are_cleaned() {
        assert_eq!(dns_name("WWW.Example.COM."), Some("www.example.com".into()));
        assert_eq!(dns_name("localhost"), None);
        assert_eq!(dns_name("4.3.2.1.in-addr.arpa"), None);
        assert_eq!(dns_name(""), None);
    }
}
