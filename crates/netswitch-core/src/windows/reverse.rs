//! Reverse DNS: the name an address answers to, for connections whose name no DNS query told.

use std::net::IpAddr;
use std::sync::Once;
use windows::Win32::Networking::WinSock::{
    socklen_t, GetNameInfoW, WSAStartup, AF_INET, AF_INET6, IN6_ADDR, IN6_ADDR_0, IN_ADDR, IN_ADDR_0, NI_MAXHOST,
    NI_NAMEREQD, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6, WSADATA,
};

fn start_winsock() {
    static START: Once = Once::new();
    START.call_once(|| unsafe {
        let mut data = WSADATA::default();
        let _ = WSAStartup(0x0202, &mut data);
    });
}

/// The PTR name of an address, or None when it has none or the lookup fails. Blocks for as long as
/// the DNS server takes, so call it off the monitor's thread.
pub fn lookup(ip: IpAddr) -> Option<String> {
    start_winsock();
    let mut host = [0u16; NI_MAXHOST as usize];
    let ret = unsafe {
        match ip {
            IpAddr::V4(v4) => {
                let addr = SOCKADDR_IN {
                    sin_family: AF_INET,
                    sin_port: 0,
                    sin_addr: IN_ADDR {
                        S_un: IN_ADDR_0 {
                            S_addr: u32::from_ne_bytes(v4.octets()),
                        },
                    },
                    sin_zero: [0; 8],
                };
                GetNameInfoW(
                    &addr as *const _ as *const SOCKADDR,
                    socklen_t(std::mem::size_of::<SOCKADDR_IN>() as i32),
                    Some(&mut host),
                    None,
                    NI_NAMEREQD as i32,
                )
            }
            IpAddr::V6(v6) => {
                let addr = SOCKADDR_IN6 {
                    sin6_family: AF_INET6,
                    sin6_addr: IN6_ADDR {
                        u: IN6_ADDR_0 { Byte: v6.octets() },
                    },
                    ..Default::default()
                };
                GetNameInfoW(
                    &addr as *const _ as *const SOCKADDR,
                    socklen_t(std::mem::size_of::<SOCKADDR_IN6>() as i32),
                    Some(&mut host),
                    None,
                    NI_NAMEREQD as i32,
                )
            }
        }
    };
    if ret != 0 {
        return None;
    }
    let len = host.iter().position(|&c| c == 0).unwrap_or(host.len());
    let name = String::from_utf16_lossy(&host[..len])
        .trim_end_matches('.')
        .to_lowercase();
    // Some resolvers answer with the address itself; that is no name.
    (!name.is_empty() && name.parse::<IpAddr>().is_err()).then_some(name)
}
