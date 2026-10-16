//! Turns transfers and socket tables into per-app and per-connection rates and totals.
//!
//! The aggregator holds no clock and no system handle: the monitor feeds it what the traffic
//! source, the connection tables and the network adapters reported, and calls `tick` once a
//! second. That keeps every rule here testable on any system.

use crate::model::{AppIdentity, AppStat, ConnectionStat, InOut, Protocol, Snapshot, Socket, TcpState, Transfer};
use crate::net::{canonical, is_local_network, service};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::net::IpAddr;

/// Seconds of per-app history the window draws.
pub const APP_HISTORY: usize = 60;
/// Seconds of history for all traffic.
pub const TOTAL_HISTORY: usize = 300;
/// A UDP exchange counts as a connection for this long after its last byte.
const UDP_ACTIVE_MS: u64 = 30_000;
/// A closed connection stays in the list for this long after its last byte.
const CLOSED_VISIBLE_MS: u64 = 5 * 60_000;
/// The connection list holds at most this many rows, busiest first.
const MAX_CONNECTIONS: usize = 1000;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct FlowKey {
    app: String,
    protocol: Protocol,
    remote_ip: IpAddr,
    remote_port: u16,
    local_port: u16,
}

#[derive(Clone, Debug, Default)]
struct Flow {
    tick: InOut,
    rate: InOut,
    total: InOut,
    state: Option<TcpState>,
    open: bool,
    last_active: Option<u64>,
}

#[derive(Clone, Debug)]
struct App {
    identity: AppIdentity,
    pids: BTreeSet<u32>,
    tick: InOut,
    rate: InOut,
    total: InOut,
    history: VecDeque<InOut>,
    listening: u32,
    first_seen: u64,
    last_active: Option<u64>,
}

impl App {
    fn new(identity: AppIdentity, now: u64) -> Self {
        Self {
            identity,
            pids: BTreeSet::new(),
            tick: InOut::default(),
            rate: InOut::default(),
            total: InOut::default(),
            history: VecDeque::with_capacity(APP_HISTORY),
            listening: 0,
            first_seen: now,
            last_active: None,
        }
    }
}

#[derive(Debug)]
pub struct Aggregator {
    per_app: bool,
    apps: HashMap<String, App>,
    flows: HashMap<FlowKey, Flow>,
    hosts: HashMap<IpAddr, String>,
    requested_names: HashSet<IpAddr>,
    adapter_last: Option<InOut>,
    adapter_tick: Option<InOut>,
    total: InOut,
    rate: InOut,
    history: VecDeque<InOut>,
}

impl Aggregator {
    /// `per_app`: whether a traffic source reports transfers per process.
    pub fn new(per_app: bool) -> Self {
        Self {
            per_app,
            apps: HashMap::new(),
            flows: HashMap::new(),
            hosts: HashMap::new(),
            requested_names: HashSet::new(),
            adapter_last: None,
            adapter_tick: None,
            total: InOut::default(),
            rate: InOut::default(),
            history: VecDeque::with_capacity(TOTAL_HISTORY),
        }
    }

    pub fn per_app(&self) -> bool {
        self.per_app
    }

    pub fn set_per_app(&mut self, per_app: bool) {
        self.per_app = per_app;
    }

    fn app(&mut self, identity: &AppIdentity, now: u64) -> &mut App {
        let app = self
            .apps
            .entry(identity.key.clone())
            .or_insert_with(|| App::new(identity.clone(), now));
        // A program's description, read later, replaces the file name; a failed read never undoes it.
        if identity.path.is_some()
            && (app.identity.path.is_none() || (app.identity.has_default_name() && !identity.has_default_name()))
        {
            app.identity = identity.clone();
        }
        app
    }

    /// Bytes the traffic source saw a process send or receive.
    pub fn record(&mut self, identity: &AppIdentity, transfer: &Transfer, now: u64) {
        let app = self.app(identity, now);
        app.pids.insert(transfer.pid);
        app.tick.add(transfer.direction, transfer.bytes);
        app.total.add(transfer.direction, transfer.bytes);
        app.last_active = Some(now);

        let key = FlowKey {
            app: identity.key.clone(),
            protocol: transfer.protocol,
            remote_ip: canonical(transfer.remote.ip()),
            remote_port: transfer.remote.port(),
            local_port: transfer.local.port(),
        };
        let flow = self.flows.entry(key).or_default();
        flow.tick.add(transfer.direction, transfer.bytes);
        flow.total.add(transfer.direction, transfer.bytes);
        flow.last_active = Some(now);
    }

    /// The sockets open right now, each with the app of its process.
    pub fn observe(&mut self, sockets: &[(AppIdentity, Socket)], now: u64) {
        for app in self.apps.values_mut() {
            app.listening = 0;
            app.pids.clear();
        }
        for flow in self.flows.values_mut() {
            flow.open = false;
            flow.state = None;
        }
        for (identity, socket) in sockets {
            let app = self.app(identity, now);
            app.pids.insert(socket.pid);
            let remote = socket.remote.filter(|r| !r.ip().is_unspecified() && r.port() != 0);
            let Some(remote) = remote else {
                // A listening TCP socket, or a UDP socket, which has no fixed remote end.
                if socket.protocol == Protocol::Tcp && socket.state == Some(TcpState::Listen)
                    || socket.protocol == Protocol::Udp
                {
                    app.listening += 1;
                }
                continue;
            };
            let key = FlowKey {
                app: identity.key.clone(),
                protocol: socket.protocol,
                remote_ip: canonical(remote.ip()),
                remote_port: remote.port(),
                local_port: socket.local.port(),
            };
            let flow = self.flows.entry(key).or_default();
            flow.open = true;
            flow.state = socket.state;
        }
    }

    /// The byte counters of the network adapters, summed. They only grow, except when an adapter
    /// goes away; then the next reading starts over from the new sum.
    pub fn adapters(&mut self, counters: InOut) {
        if let Some(last) = self.adapter_last {
            if counters.down >= last.down && counters.up >= last.up {
                let delta = InOut::new(counters.down - last.down, counters.up - last.up);
                let tick = self.adapter_tick.get_or_insert_with(InOut::default);
                tick.down += delta.down;
                tick.up += delta.up;
            }
        }
        self.adapter_last = Some(counters);
    }

    /// The name an address answered to: the name an app asked DNS for, or a reverse DNS name.
    pub fn set_host(&mut self, ip: IpAddr, name: String) {
        self.hosts.insert(canonical(ip), name);
    }

    /// Addresses of connections with no name yet, each returned once.
    pub fn take_unnamed(&mut self) -> Vec<IpAddr> {
        let mut out = Vec::new();
        for key in self.flows.keys() {
            if !self.hosts.contains_key(&key.remote_ip) && self.requested_names.insert(key.remote_ip) {
                out.push(key.remote_ip);
            }
        }
        out
    }

    /// Closes one interval: what came in since the last tick becomes the rate.
    pub fn tick(&mut self, now: u64, elapsed_ms: u64) {
        let elapsed_ms = elapsed_ms.max(1);
        let per_second = |b: InOut| InOut::new(b.down * 1000 / elapsed_ms, b.up * 1000 / elapsed_ms);

        let mut app_sum = InOut::default();
        for app in self.apps.values_mut() {
            app.rate = per_second(app.tick);
            app_sum.down += app.tick.down;
            app_sum.up += app.tick.up;
            app.tick = InOut::default();
            if app.history.len() == APP_HISTORY {
                app.history.pop_front();
            }
            app.history.push_back(app.rate);
        }
        for flow in self.flows.values_mut() {
            flow.rate = per_second(flow.tick);
            flow.tick = InOut::default();
        }

        // The adapters see all traffic, also what no process owns, so they give the total when read.
        let interval = self.adapter_tick.take().unwrap_or(app_sum);
        self.rate = per_second(interval);
        self.total.down += interval.down;
        self.total.up += interval.up;
        if self.history.len() == TOTAL_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(self.rate);

        self.flows.retain(|_, flow| {
            flow.open
                || flow
                    .last_active
                    .is_some_and(|t| now.saturating_sub(t) < CLOSED_VISIBLE_MS)
        });
    }

    /// `is_blocked` and `is_protected` get a lowercased program path.
    pub fn snapshot(
        &self,
        now: u64,
        is_blocked: &dyn Fn(&str) -> bool,
        is_protected: &dyn Fn(&str) -> bool,
    ) -> Snapshot {
        let mut per_app_flows: HashMap<&str, (u32, HashSet<IpAddr>)> = HashMap::new();
        for (key, flow) in &self.flows {
            let entry = per_app_flows.entry(key.app.as_str()).or_default();
            if is_connection(key, flow, now) {
                entry.0 += 1;
            }
            entry.1.insert(key.remote_ip);
        }

        let mut apps: Vec<AppStat> = self
            .apps
            .values()
            .map(|app| {
                let (connections, hosts) = per_app_flows
                    .get(app.identity.key.as_str())
                    .map(|(c, h)| (*c, h.len() as u32))
                    .unwrap_or((0, 0));
                AppStat {
                    key: app.identity.key.clone(),
                    name: app.identity.name.clone(),
                    path: app.identity.path.clone(),
                    pids: app.pids.iter().copied().collect(),
                    rate: app.rate,
                    total: app.total,
                    history: app.history.iter().copied().collect(),
                    connections,
                    listening: app.listening,
                    hosts,
                    first_seen: app.first_seen,
                    last_active: app.last_active,
                    blocked: app
                        .identity
                        .path
                        .as_deref()
                        .is_some_and(|p| is_blocked(&p.to_lowercase())),
                    protected: app
                        .identity
                        .path
                        .as_deref()
                        .is_some_and(|p| is_protected(&p.to_lowercase())),
                }
            })
            .collect();
        apps.sort_by(|a, b| {
            b.rate
                .sum()
                .cmp(&a.rate.sum())
                .then(b.total.sum().cmp(&a.total.sum()))
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        let mut connections: Vec<ConnectionStat> = self
            .flows
            .iter()
            .map(|(key, flow)| {
                let app = self.apps.get(&key.app);
                ConnectionStat {
                    app: key.app.clone(),
                    app_name: app.map(|a| a.identity.name.clone()).unwrap_or_default(),
                    protocol: key.protocol,
                    local_port: key.local_port,
                    remote_ip: key.remote_ip,
                    remote_port: key.remote_port,
                    host: self.hosts.get(&key.remote_ip).cloned(),
                    service: service(key.protocol, key.remote_port),
                    local_network: is_local_network(key.remote_ip),
                    state: flow.state,
                    open: is_connection(key, flow, now),
                    rate: flow.rate,
                    total: flow.total,
                    last_active: flow.last_active,
                }
            })
            .collect();
        connections.sort_by(|a, b| {
            b.open
                .cmp(&a.open)
                .then(b.rate.sum().cmp(&a.rate.sum()))
                .then(b.total.sum().cmp(&a.total.sum()))
                .then(a.app_name.cmp(&b.app_name))
                .then(a.remote_ip.cmp(&b.remote_ip))
                .then(a.remote_port.cmp(&b.remote_port))
        });
        connections.truncate(MAX_CONNECTIONS);

        Snapshot {
            at: now,
            per_app: self.per_app,
            rate: self.rate,
            total: self.total,
            history: self.history.iter().copied().collect(),
            apps,
            connections,
        }
    }
}

/// An open TCP connection, or a UDP exchange with bytes in the last half minute.
fn is_connection(key: &FlowKey, flow: &Flow, now: u64) -> bool {
    match key.protocol {
        Protocol::Tcp => flow.open,
        Protocol::Udp => flow.open || flow.last_active.is_some_and(|t| now.saturating_sub(t) < UDP_ACTIVE_MS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Direction;
    use std::net::SocketAddr;

    fn app(path: &str) -> AppIdentity {
        AppIdentity::from_path(path, None)
    }

    fn transfer(pid: u32, direction: Direction, bytes: u64, remote: &str) -> Transfer {
        Transfer {
            pid,
            protocol: Protocol::Tcp,
            direction,
            bytes,
            local: "192.168.1.10:50000".parse().unwrap(),
            remote: remote.parse().unwrap(),
        }
    }

    fn socket(pid: u32, protocol: Protocol, local: &str, remote: Option<&str>, state: Option<TcpState>) -> Socket {
        Socket {
            pid,
            protocol,
            local: local.parse().unwrap(),
            remote: remote.map(|r| r.parse::<SocketAddr>().unwrap()),
            state,
        }
    }

    fn never_blocked(_: &str) -> bool {
        false
    }

    #[test]
    fn bytes_of_one_second_become_the_rate() {
        let chrome = app("C:\\Chrome\\chrome.exe");
        let mut agg = Aggregator::new(true);
        agg.record(&chrome, &transfer(10, Direction::In, 3000, "142.250.1.1:443"), 1_000);
        agg.record(&chrome, &transfer(11, Direction::In, 1000, "142.250.1.1:443"), 1_500);
        agg.record(&chrome, &transfer(10, Direction::Out, 500, "142.250.1.1:443"), 1_600);
        agg.tick(2_000, 1_000);

        let snap = agg.snapshot(2_000, &never_blocked, &never_blocked);
        let stat = &snap.apps[0];
        assert_eq!(stat.name, "chrome");
        assert_eq!(stat.pids, vec![10, 11]);
        assert_eq!(stat.rate, InOut::new(4000, 500));
        assert_eq!(stat.total, InOut::new(4000, 500));
        assert_eq!(
            snap.rate,
            InOut::new(4000, 500),
            "without adapter counters the total is the sum of apps"
        );

        agg.tick(3_000, 1_000);
        let snap = agg.snapshot(3_000, &never_blocked, &never_blocked);
        assert_eq!(snap.apps[0].rate, InOut::default(), "a quiet second has no rate");
        assert_eq!(snap.apps[0].total, InOut::new(4000, 500), "totals stay");
        assert_eq!(snap.apps[0].history, vec![InOut::new(4000, 500), InOut::default()]);
    }

    #[test]
    fn a_longer_interval_is_divided_by_its_length() {
        let a = app("C:\\a.exe");
        let mut agg = Aggregator::new(true);
        agg.record(&a, &transfer(1, Direction::In, 3000, "1.1.1.1:443"), 0);
        agg.tick(1_500, 1_500);
        assert_eq!(
            agg.snapshot(1_500, &never_blocked, &never_blocked).apps[0].rate,
            InOut::new(2000, 0)
        );
    }

    #[test]
    fn adapter_counters_give_the_total_rate() {
        let mut agg = Aggregator::new(false);
        agg.adapters(InOut::new(1_000_000, 50_000));
        agg.tick(1_000, 1_000);
        assert_eq!(
            agg.snapshot(1_000, &never_blocked, &never_blocked).rate,
            InOut::default(),
            "the first reading is only a baseline"
        );

        agg.adapters(InOut::new(1_250_000, 60_000));
        agg.tick(2_000, 1_000);
        let snap = agg.snapshot(2_000, &never_blocked, &never_blocked);
        assert_eq!(snap.rate, InOut::new(250_000, 10_000));
        assert_eq!(snap.total, InOut::new(250_000, 10_000));

        // An adapter went away: the sum dropped. That reading is a new baseline, not negative traffic.
        agg.adapters(InOut::new(100, 100));
        agg.tick(3_000, 1_000);
        assert_eq!(
            agg.snapshot(3_000, &never_blocked, &never_blocked).rate,
            InOut::default()
        );
        agg.adapters(InOut::new(1100, 600));
        agg.tick(4_000, 1_000);
        assert_eq!(
            agg.snapshot(4_000, &never_blocked, &never_blocked).rate,
            InOut::new(1000, 500)
        );
    }

    #[test]
    fn histories_keep_their_length() {
        let a = app("C:\\a.exe");
        let mut agg = Aggregator::new(true);
        for second in 1..=400u64 {
            agg.record(&a, &transfer(1, Direction::In, second, "1.1.1.1:443"), second * 1000);
            agg.tick(second * 1000, 1000);
        }
        let snap = agg.snapshot(400_000, &never_blocked, &never_blocked);
        assert_eq!(snap.history.len(), TOTAL_HISTORY);
        assert_eq!(snap.history.last().unwrap().down, 400);
        assert_eq!(snap.apps[0].history.len(), APP_HISTORY);
        assert_eq!(snap.apps[0].history[0].down, 341);
    }

    #[test]
    fn sockets_count_connections_and_listeners() {
        let server = app("C:\\server.exe");
        let mut agg = Aggregator::new(true);
        agg.observe(
            &[
                (
                    server.clone(),
                    socket(
                        7,
                        Protocol::Tcp,
                        "0.0.0.0:8080",
                        Some("0.0.0.0:0"),
                        Some(TcpState::Listen),
                    ),
                ),
                (
                    server.clone(),
                    socket(
                        7,
                        Protocol::Tcp,
                        "10.0.0.2:8080",
                        Some("10.0.0.9:51000"),
                        Some(TcpState::Established),
                    ),
                ),
                (
                    server.clone(),
                    socket(
                        8,
                        Protocol::Tcp,
                        "10.0.0.2:50001",
                        Some("93.184.216.34:443"),
                        Some(TcpState::Established),
                    ),
                ),
                (server.clone(), socket(8, Protocol::Udp, "0.0.0.0:5353", None, None)),
            ],
            1_000,
        );
        let snap = agg.snapshot(1_000, &never_blocked, &never_blocked);
        let stat = &snap.apps[0];
        assert_eq!(stat.pids, vec![7, 8]);
        assert_eq!(stat.listening, 2);
        assert_eq!(stat.connections, 2);
        assert_eq!(stat.hosts, 2);
        assert_eq!(snap.connections.len(), 2);
        assert!(snap
            .connections
            .iter()
            .all(|c| c.open && c.state == Some(TcpState::Established)));
        let internet = snap.connections.iter().find(|c| c.remote_port == 443).unwrap();
        assert!(!internet.local_network);
        assert_eq!(internet.service, Some("https"));
    }

    #[test]
    fn closed_connections_stay_visible_for_five_minutes() {
        let a = app("C:\\a.exe");
        let mut agg = Aggregator::new(true);
        agg.observe(
            &[(
                a.clone(),
                socket(
                    1,
                    Protocol::Tcp,
                    "10.0.0.2:5000",
                    Some("1.1.1.1:443"),
                    Some(TcpState::Established),
                ),
            )],
            0,
        );
        agg.record(
            &a,
            &Transfer {
                local: "10.0.0.2:5000".parse().unwrap(),
                ..transfer(1, Direction::In, 10, "1.1.1.1:443")
            },
            0,
        );
        agg.tick(1_000, 1_000);

        agg.observe(&[], 2_000);
        agg.tick(2_000, 1_000);
        let snap = agg.snapshot(2_000, &never_blocked, &never_blocked);
        assert_eq!(snap.connections.len(), 1);
        assert!(!snap.connections[0].open);
        assert_eq!(snap.apps[0].connections, 0);

        agg.tick(CLOSED_VISIBLE_MS + 1, 1_000);
        assert!(agg
            .snapshot(CLOSED_VISIBLE_MS + 1, &never_blocked, &never_blocked)
            .connections
            .is_empty());
    }

    #[test]
    fn udp_counts_as_a_connection_while_bytes_flow() {
        let a = app("C:\\game.exe");
        let mut agg = Aggregator::new(true);
        let udp = Transfer {
            protocol: Protocol::Udp,
            ..transfer(3, Direction::In, 100, "203.0.113.5:27015")
        };
        agg.record(&a, &udp, 1_000);
        agg.tick(1_000, 1_000);
        assert_eq!(
            agg.snapshot(1_000, &never_blocked, &never_blocked).apps[0].connections,
            1
        );
        assert_eq!(
            agg.snapshot(1_000 + UDP_ACTIVE_MS, &never_blocked, &never_blocked).apps[0].connections,
            0
        );
    }

    #[test]
    fn mapped_ipv6_addresses_join_their_ipv4_flow() {
        let a = app("C:\\a.exe");
        let mut agg = Aggregator::new(true);
        agg.record(&a, &transfer(1, Direction::In, 10, "1.2.3.4:443"), 0);
        agg.record(&a, &transfer(1, Direction::In, 10, "[::ffff:1.2.3.4]:443"), 0);
        agg.tick(1_000, 1_000);
        let snap = agg.snapshot(1_000, &never_blocked, &never_blocked);
        assert_eq!(snap.connections.len(), 1);
        assert_eq!(snap.connections[0].total.down, 20);
    }

    #[test]
    fn names_are_asked_for_once_and_shown_when_known() {
        let a = app("C:\\a.exe");
        let mut agg = Aggregator::new(true);
        agg.record(&a, &transfer(1, Direction::In, 10, "1.2.3.4:443"), 0);
        assert_eq!(agg.take_unnamed(), vec!["1.2.3.4".parse::<IpAddr>().unwrap()]);
        assert!(agg.take_unnamed().is_empty());
        agg.set_host("::ffff:1.2.3.4".parse().unwrap(), "example.com".into());
        assert_eq!(
            agg.snapshot(0, &never_blocked, &never_blocked).connections[0]
                .host
                .as_deref(),
            Some("example.com")
        );
    }

    #[test]
    fn busiest_apps_come_first_and_blocked_and_protected_ones_are_marked() {
        let quiet = app("C:\\quiet.exe");
        let busy = app("C:\\Busy.exe");
        let mut agg = Aggregator::new(true);
        agg.record(&quiet, &transfer(1, Direction::In, 10, "1.1.1.1:443"), 0);
        agg.record(&busy, &transfer(2, Direction::In, 9000, "1.1.1.1:443"), 0);
        agg.tick(1_000, 1_000);
        let snap = agg.snapshot(1_000, &|path| path == "c:\\busy.exe", &|path| path == "c:\\quiet.exe");
        assert_eq!(
            snap.apps.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["Busy", "quiet"]
        );
        assert!(snap.apps[0].blocked);
        assert!(!snap.apps[1].blocked);
        assert!(!snap.apps[0].protected);
        assert!(snap.apps[1].protected);
    }

    #[test]
    fn a_later_description_replaces_the_file_name() {
        let mut agg = Aggregator::new(true);
        agg.record(
            &AppIdentity::from_path("C:\\Chrome\\chrome.exe", None),
            &transfer(1, Direction::In, 1, "1.1.1.1:443"),
            0,
        );
        agg.record(
            &AppIdentity::from_path("C:\\Chrome\\chrome.exe", Some("Google Chrome".into())),
            &transfer(1, Direction::In, 1, "1.1.1.1:443"),
            0,
        );
        assert_eq!(
            agg.snapshot(0, &never_blocked, &never_blocked).apps[0].name,
            "Google Chrome"
        );
    }
}
