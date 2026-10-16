//! Tests against a real Windows system: sockets, adapters, program names, the traffic trace and
//! the firewall. The trace and the firewall need administrator rights, so these tests are ignored
//! by default; CI runs them on a Windows runner (which is elevated) with `--include-ignored`.
#![cfg(windows)]

use netswitch_core::blocklist::blocklist;
use netswitch_core::model::{Direction, Protocol, TcpState};
use netswitch_core::windows::{firewall, iphlp, process, reverse, traffic::TrafficSource};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "reads the system; run with --include-ignored on Windows"]
fn the_tcp_table_lists_a_listener_of_this_process() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let sockets = iphlp::sockets().unwrap();
    let found = sockets
        .iter()
        .find(|s| s.protocol == Protocol::Tcp && s.local.port() == port)
        .expect("the listener is in the table");
    assert_eq!(found.pid, std::process::id());
    assert_eq!(found.state, Some(TcpState::Listen));
}

#[test]
#[ignore = "reads the system; run with --include-ignored on Windows"]
fn the_udp_table_lists_a_socket_of_this_process() {
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();
    let sockets = iphlp::sockets().unwrap();
    assert!(sockets
        .iter()
        .any(|s| s.protocol == Protocol::Udp && s.local.port() == port && s.pid == std::process::id()));
}

#[test]
#[ignore = "reads the system; run with --include-ignored on Windows"]
fn adapter_counters_can_be_read() {
    let first = iphlp::adapter_counters().unwrap();
    let second = iphlp::adapter_counters().unwrap();
    assert!(second.down >= first.down && second.up >= first.up);
}

#[test]
#[ignore = "reads the system; run with --include-ignored on Windows"]
fn program_path_and_description_of_a_process() {
    let path = process::program_path(std::process::id()).expect("this process's path");
    assert!(path.eq_ignore_ascii_case(&std::env::current_exe().unwrap().to_string_lossy()));
    assert_eq!(process::program_path(4), None, "the System process has no program file");

    let cmd = std::env::var("ComSpec").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".into());
    assert_eq!(
        process::file_description(&cmd).as_deref(),
        Some("Windows Command Processor")
    );
}

#[test]
#[ignore = "needs administrator rights and the network; run with --include-ignored on Windows"]
fn the_trace_counts_traffic_per_process_with_the_ends_the_right_way_round() {
    let source = TrafficSource::start().expect("the trace starts as administrator");
    thread::sleep(Duration::from_secs(2));
    let _ = source.take_transfers();

    // Loopback: this process is both ends, so the ports tell the ends apart.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server_port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        let mut buf = vec![0u8; 200_000];
        let mut got = 0;
        while got < buf.len() {
            let n = conn.read(&mut buf[got..]).unwrap();
            if n == 0 {
                break;
            }
            got += n;
        }
        conn.write_all(&[7u8; 50_000]).unwrap();
    });
    let mut client = TcpStream::connect(("127.0.0.1", server_port)).unwrap();
    let client_port = client.local_addr().unwrap().port();
    client.write_all(&[1u8; 200_000]).unwrap();
    let mut reply = vec![0u8; 50_000];
    client.read_exact(&mut reply).unwrap();
    server.join().unwrap();

    // The internet, by name, so the DNS client reports the lookup too.
    let mut web = TcpStream::connect("example.com:80").unwrap();
    let web_remote = web.peer_addr().unwrap();
    web.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut page = Vec::new();
    web.read_to_end(&mut page).unwrap();
    assert!(!page.is_empty());

    thread::sleep(Duration::from_secs(3));
    let transfers = source.take_transfers();
    let names = source.take_names();
    let me = std::process::id();
    let mine: Vec<_> = transfers.iter().filter(|t| t.pid == me).collect();
    assert!(!mine.is_empty(), "transfers of this process are counted: {transfers:?}");

    let sent_to_server: u64 = mine
        .iter()
        .filter(|t| t.direction == Direction::Out && t.remote.port() == server_port && t.local.port() == client_port)
        .map(|t| t.bytes)
        .sum();
    assert!(
        sent_to_server >= 200_000,
        "the client's sends have the server as remote end: {mine:?}"
    );
    let received_by_server: u64 = mine
        .iter()
        .filter(|t| t.direction == Direction::In && t.local.port() == server_port && t.remote.port() == client_port)
        .map(|t| t.bytes)
        .sum();
    assert!(
        received_by_server >= 200_000,
        "the server's receives have the client as remote end: {mine:?}"
    );

    let from_web: u64 = mine
        .iter()
        .filter(|t| {
            t.protocol == Protocol::Tcp
                && t.direction == Direction::In
                && t.remote.port() == 80
                && t.remote.ip() == web_remote.ip()
        })
        .map(|t| t.bytes)
        .sum();
    assert!(
        from_web >= page.len() as u64 / 2,
        "the page came in from example.com:80: {mine:?}"
    );

    let web_ips: Vec<_> = "example.com:80".to_socket_addrs().unwrap().map(|a| a.ip()).collect();
    assert!(
        names
            .iter()
            .any(|(ip, name)| name == "example.com" && web_ips.contains(ip)),
        "the DNS client named example.com's address: {names:?}"
    );
}

#[test]
#[ignore = "needs the network; run with --include-ignored on Windows"]
fn reverse_dns_names_a_public_resolver() {
    let name = reverse::lookup("8.8.8.8".parse().unwrap()).expect("8.8.8.8 has a PTR name");
    assert_eq!(name, "dns.google");
    assert_eq!(
        reverse::lookup("192.0.2.1".parse().unwrap()),
        None,
        "a documentation address has no name"
    );
}

#[test]
#[ignore = "changes the firewall; needs administrator rights; run with --include-ignored on Windows"]
fn blocking_and_unblocking_a_program() {
    let program = "C:\\netswitch-test\\does-not-exist.exe";
    let _ = firewall::unblock_program(program);

    firewall::block(program, "netSwitch test - app").unwrap();
    firewall::block(program, "netSwitch test - app").unwrap(); // a second call adds nothing

    let rules = firewall::read_rules().unwrap();
    let mine: Vec<_> = rules.iter().filter(|r| r.program.as_deref() == Some(program)).collect();
    assert_eq!(mine.len(), 2, "one inbound and one outbound rule: {mine:?}");
    assert!(mine.iter().all(|r| r.block && r.enabled));
    let list = blocklist(&rules);
    assert!(list.is_blocked(&program.to_lowercase()));
    let app = list
        .apps
        .iter()
        .find(|a| a.programs.iter().any(|p| p == program))
        .unwrap();
    assert_eq!(app.name, "netSwitch test app");
    assert!(app.complete);

    assert_eq!(firewall::unblock_program(program).unwrap(), 2);
    let after = firewall::read_rules().unwrap();
    assert!(!after.iter().any(|r| r.program.as_deref() == Some(program)));
}

#[test]
#[ignore = "writes a file; needs administrator rights; run with --include-ignored on Windows"]
fn the_firewall_policy_can_be_exported() {
    let file = std::env::temp_dir().join(format!("netswitch-test-{}.wfw", std::process::id()));
    let _ = std::fs::remove_file(&file);
    firewall::export_policy(&file).unwrap();
    assert!(std::fs::metadata(&file).unwrap().len() > 0);
    let _ = std::fs::remove_file(&file);
}
