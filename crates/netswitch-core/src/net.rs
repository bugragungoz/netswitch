//! Addresses and ports: which ones stay on this computer or network, and what common ports are for.

use crate::model::Protocol;
use std::net::IpAddr;

/// True for loopback, private, link-local, unspecified and multicast addresses: traffic that does
/// not leave for the internet.
pub fn is_local_network(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified() || v4.is_multicast() || v4.is_broadcast()
                // 100.64.0.0/10, carrier-grade NAT
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 64)
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_local_network(IpAddr::V4(v4));
            }
            let first = v6.segments()[0];
            v6.is_loopback() || v6.is_unspecified() || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00 // unique local fc00::/7
                || (first & 0xffc0) == 0xfe80 // link-local fe80::/10
        }
    }
}

/// An IPv4 address written as IPv6 (::ffff:a.b.c.d) is shown and grouped as IPv4.
pub fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip),
        v4 => v4,
    }
}

/// The well-known service of a port, for the ports people meet in a connection list.
pub fn service(protocol: Protocol, port: u16) -> Option<&'static str> {
    Some(match (protocol, port) {
        (_, 53) => "dns",
        (Protocol::Tcp, 20 | 21) => "ftp",
        (Protocol::Tcp, 22) => "ssh",
        (Protocol::Tcp, 23) => "telnet",
        (Protocol::Tcp, 25 | 465 | 587) => "smtp",
        (Protocol::Udp, 67 | 68) => "dhcp",
        (Protocol::Tcp, 80 | 8080) => "http",
        (Protocol::Tcp, 110 | 995) => "pop3",
        (Protocol::Udp, 123) => "ntp",
        (Protocol::Udp, 137 | 138) | (Protocol::Tcp, 139) => "netbios",
        (Protocol::Tcp, 143 | 993) => "imap",
        (Protocol::Udp, 161 | 162) => "snmp",
        (Protocol::Tcp, 443 | 8443) => "https",
        (Protocol::Udp, 443) => "quic",
        (Protocol::Tcp, 445) => "smb",
        (Protocol::Udp, 500 | 4500) => "ipsec",
        (Protocol::Tcp, 853) => "dns-over-tls",
        (Protocol::Udp, 1900) => "ssdp",
        (Protocol::Tcp, 1883 | 8883) => "mqtt",
        (Protocol::Tcp, 3389) | (Protocol::Udp, 3389) => "rdp",
        (Protocol::Udp, 3478 | 19302) => "stun",
        (Protocol::Udp, 5353) => "mdns",
        (Protocol::Udp, 5355) => "llmnr",
        (Protocol::Tcp, 5228) => "google-play",
        (Protocol::Tcp, 5222 | 5223) => "xmpp",
        (Protocol::Tcp, 6881..=6889) | (Protocol::Udp, 6881..=6889) => "bittorrent",
        (Protocol::Tcp, 7680) => "delivery-optimization",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn local_network_addresses() {
        for local in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.1.10",
            "172.16.0.1",
            "169.254.1.1",
            "0.0.0.0",
            "224.0.0.251",
            "100.64.0.1",
            "::1",
            "fe80::1",
            "fd00::1",
            "ff02::fb",
            "::ffff:192.168.1.1",
        ] {
            assert!(is_local_network(ip(local)), "{local} is local");
        }
        for internet in [
            "8.8.8.8",
            "142.250.187.206",
            "100.128.0.1",
            "2a00:1450:4017:80b::200e",
            "::ffff:8.8.8.8",
        ] {
            assert!(!is_local_network(ip(internet)), "{internet} is on the internet");
        }
    }

    #[test]
    fn mapped_addresses_are_shown_as_ipv4() {
        assert_eq!(canonical(ip("::ffff:1.2.3.4")), ip("1.2.3.4"));
        assert_eq!(canonical(ip("2001:db8::1")), ip("2001:db8::1"));
    }

    #[test]
    fn services_depend_on_the_protocol() {
        assert_eq!(service(Protocol::Tcp, 443), Some("https"));
        assert_eq!(service(Protocol::Udp, 443), Some("quic"));
        assert_eq!(service(Protocol::Udp, 53), Some("dns"));
        assert_eq!(service(Protocol::Tcp, 53), Some("dns"));
        assert_eq!(service(Protocol::Tcp, 50000), None);
    }
}
