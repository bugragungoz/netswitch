//! One step of the monitor: read the system, feed the aggregator, close the second.
//!
//! The system is behind the `Source` trait, so the loop is the same on Windows and in tests.

use crate::aggregate::Aggregator;
use crate::apps::AppResolver;
use crate::blocklist::{blocklist, Blocklist, FirewallRule};
use crate::model::{AppIdentity, InOut, Snapshot, Socket, Transfer};
use crate::net::is_local_network;
use crate::rules::is_protected;
use std::collections::HashSet;
use std::net::IpAddr;

/// How often the firewall rules are read again, in milliseconds. Changes netSwitch makes itself
/// are read right away.
const BLOCKLIST_EVERY_MS: u64 = 5_000;

/// What the monitor reads from the system.
pub trait Source {
    /// Transfers counted since the last call; empty when traffic per app is not measured.
    fn take_transfers(&mut self) -> Vec<Transfer>;
    /// Addresses with the names programs looked them up by, since the last call.
    fn take_names(&mut self) -> Vec<(IpAddr, String)>;
    fn sockets(&mut self) -> Result<Vec<Socket>, String>;
    /// Byte counters of the network adapters, summed.
    fn adapters(&mut self) -> Result<InOut, String>;
    fn firewall_rules(&mut self) -> Result<Vec<FirewallRule>, String>;
    fn per_app(&self) -> bool;
}

/// An app that sent or received its first bytes to or from the internet in this run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewApp {
    pub key: String,
    pub name: String,
}

pub struct Monitor<P, D>
where
    P: FnMut(u32) -> Option<String>,
    D: FnMut(&str) -> Option<String>,
{
    aggregator: Aggregator,
    resolver: AppResolver<P, D>,
    blocklist: Blocklist,
    blocklist_read_at: Option<u64>,
    last_tick: Option<u64>,
    online: HashSet<String>,
    system_root: String,
    /// The last error of each part that failed, for the status bar.
    pub errors: Vec<String>,
}

impl<P, D> Monitor<P, D>
where
    P: FnMut(u32) -> Option<String>,
    D: FnMut(&str) -> Option<String>,
{
    /// `system_root`: the Windows folder; programs inside it are never blocked.
    pub fn new(per_app: bool, system_root: &str, program_path: P, description: D) -> Self {
        Self {
            aggregator: Aggregator::new(per_app),
            resolver: AppResolver::new(program_path, description),
            blocklist: Blocklist::default(),
            blocklist_read_at: None,
            last_tick: None,
            online: HashSet::new(),
            system_root: system_root.to_string(),
            errors: Vec::new(),
        }
    }

    pub fn blocklist(&self) -> &Blocklist {
        &self.blocklist
    }

    /// Reads the firewall rules again at the next step.
    pub fn refresh_blocklist(&mut self) {
        self.blocklist_read_at = None;
    }

    /// Names learned outside the traffic source, such as from reverse DNS.
    pub fn set_host(&mut self, ip: IpAddr, name: String) {
        self.aggregator.set_host(ip, name);
    }

    /// Addresses with no name yet, each returned once, for reverse DNS.
    pub fn take_unnamed(&mut self) -> Vec<IpAddr> {
        self.aggregator.take_unnamed()
    }

    /// One second of monitoring. Returns the apps that reached the internet for the first time.
    pub fn step(&mut self, source: &mut dyn Source, now: u64) -> Vec<NewApp> {
        self.errors.clear();
        self.aggregator.set_per_app(source.per_app());
        let mut first_online = Vec::new();

        for transfer in source.take_transfers() {
            let identity = self.resolver.identity(transfer.pid, now);
            if !is_local_network(transfer.remote.ip()) && self.online.insert(identity.key.clone()) {
                first_online.push(NewApp {
                    key: identity.key.clone(),
                    name: identity.name.clone(),
                });
            }
            self.aggregator.record(&identity, &transfer, now);
        }
        for (ip, name) in source.take_names() {
            self.aggregator.set_host(ip, name);
        }

        match source.sockets() {
            Ok(sockets) => {
                let with_apps: Vec<(AppIdentity, Socket)> = sockets
                    .into_iter()
                    .map(|s| (self.resolver.identity(s.pid, now), s))
                    .collect();
                self.aggregator.observe(&with_apps, now);
            }
            Err(e) => self.errors.push(format!("sockets: {e}")),
        }
        match source.adapters() {
            Ok(counters) => self.aggregator.adapters(counters),
            Err(e) => self.errors.push(format!("adapters: {e}")),
        }

        if self
            .blocklist_read_at
            .is_none_or(|at| now.saturating_sub(at) >= BLOCKLIST_EVERY_MS)
        {
            match source.firewall_rules() {
                Ok(rules) => self.blocklist = blocklist(&rules),
                Err(e) => self.errors.push(format!("firewall: {e}")),
            }
            self.blocklist_read_at = Some(now);
        }

        let elapsed = self.last_tick.map(|t| now.saturating_sub(t)).unwrap_or(1000);
        self.aggregator.tick(now, elapsed);
        self.last_tick = Some(now);
        self.resolver.prune(now);
        first_online
    }

    pub fn snapshot(&self, now: u64) -> Snapshot {
        let blocklist = &self.blocklist;
        let root = &self.system_root;
        self.aggregator
            .snapshot(now, &|path| blocklist.is_blocked(path), &|path| {
                is_protected(path, root)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Direction, Protocol, TcpState};
    use crate::rules::{rule_name, RuleDirection};

    #[derive(Default)]
    struct FakeSource {
        transfers: Vec<Transfer>,
        names: Vec<(IpAddr, String)>,
        sockets: Vec<Socket>,
        adapters: InOut,
        rules: Vec<FirewallRule>,
        rule_reads: u32,
        fail_sockets: bool,
    }

    impl Source for FakeSource {
        fn take_transfers(&mut self) -> Vec<Transfer> {
            std::mem::take(&mut self.transfers)
        }
        fn take_names(&mut self) -> Vec<(IpAddr, String)> {
            std::mem::take(&mut self.names)
        }
        fn sockets(&mut self) -> Result<Vec<Socket>, String> {
            if self.fail_sockets {
                Err("access denied".into())
            } else {
                Ok(self.sockets.clone())
            }
        }
        fn adapters(&mut self) -> Result<InOut, String> {
            Ok(self.adapters)
        }
        fn firewall_rules(&mut self) -> Result<Vec<FirewallRule>, String> {
            self.rule_reads += 1;
            Ok(self.rules.clone())
        }
        fn per_app(&self) -> bool {
            true
        }
    }

    fn transfer(pid: u32, remote: &str, bytes: u64) -> Transfer {
        Transfer {
            pid,
            protocol: Protocol::Tcp,
            direction: Direction::In,
            bytes,
            local: "10.0.0.2:50000".parse().unwrap(),
            remote: remote.parse().unwrap(),
        }
    }

    fn monitor() -> Monitor<impl FnMut(u32) -> Option<String>, impl FnMut(&str) -> Option<String>> {
        Monitor::new(
            true,
            "C:\\Windows",
            |pid| match pid {
                10 => Some("C:\\Browser\\browser.exe".to_string()),
                20 => Some("C:\\Printer\\spooler.exe".to_string()),
                30 => Some("C:\\Windows\\System32\\svchost.exe".to_string()),
                _ => None,
            },
            |path| path.ends_with("browser.exe").then(|| "Web Browser".to_string()),
        )
    }

    #[test]
    fn a_step_turns_system_readings_into_a_snapshot() {
        let mut source = FakeSource {
            transfers: vec![transfer(10, "93.184.216.34:443", 5000)],
            names: vec![("93.184.216.34".parse().unwrap(), "example.com".into())],
            sockets: vec![Socket {
                pid: 10,
                protocol: Protocol::Tcp,
                local: "10.0.0.2:50000".parse().unwrap(),
                remote: Some("93.184.216.34:443".parse().unwrap()),
                state: Some(TcpState::Established),
            }],
            adapters: InOut::new(1000, 100),
            rules: vec![FirewallRule {
                name: rule_name("Web Browser", "browser.exe", RuleDirection::Outbound),
                program: Some("C:\\Browser\\browser.exe".into()),
                direction: RuleDirection::Outbound,
                block: true,
                enabled: true,
            }],
            ..FakeSource::default()
        };
        let mut m = monitor();
        let new_apps = m.step(&mut source, 1_000);
        assert_eq!(
            new_apps,
            vec![NewApp {
                key: "c:\\browser\\browser.exe".into(),
                name: "Web Browser".into()
            }]
        );

        let snap = m.snapshot(1_000);
        let app = &snap.apps[0];
        assert_eq!(app.name, "Web Browser");
        assert_eq!(app.rate.down, 5000);
        assert!(app.blocked);
        assert_eq!(app.connections, 1);
        assert_eq!(snap.connections[0].host.as_deref(), Some("example.com"));
        assert!(m.errors.is_empty());
    }

    #[test]
    fn programs_in_the_windows_folder_are_marked_protected() {
        let mut m = monitor();
        let mut source = FakeSource {
            transfers: vec![transfer(30, "1.1.1.1:443", 10), transfer(10, "1.1.1.1:443", 5)],
            ..FakeSource::default()
        };
        m.step(&mut source, 1_000);
        let snap = m.snapshot(1_000);
        assert!(snap.apps.iter().find(|a| a.name == "svchost").unwrap().protected);
        assert!(!snap.apps.iter().find(|a| a.name == "Web Browser").unwrap().protected);
    }

    #[test]
    fn an_app_is_new_only_once_and_only_for_the_internet() {
        let mut m = monitor();
        let mut source = FakeSource {
            transfers: vec![transfer(20, "192.168.1.50:9100", 10)],
            ..FakeSource::default()
        };
        assert!(
            m.step(&mut source, 1_000).is_empty(),
            "a printer on the local network is not the internet"
        );
        source.transfers = vec![transfer(10, "1.1.1.1:443", 10)];
        assert_eq!(m.step(&mut source, 2_000).len(), 1);
        source.transfers = vec![transfer(10, "1.1.1.1:443", 10)];
        assert!(m.step(&mut source, 3_000).is_empty());
    }

    #[test]
    fn firewall_rules_are_read_every_few_seconds_or_when_asked() {
        let mut m = monitor();
        let mut source = FakeSource::default();
        m.step(&mut source, 1_000);
        m.step(&mut source, 2_000);
        assert_eq!(source.rule_reads, 1);
        m.step(&mut source, 1_000 + BLOCKLIST_EVERY_MS);
        assert_eq!(source.rule_reads, 2);
        m.refresh_blocklist();
        m.step(&mut source, 2_000 + BLOCKLIST_EVERY_MS);
        assert_eq!(source.rule_reads, 3);
    }

    #[test]
    fn a_failing_part_is_reported_and_the_rest_goes_on() {
        let mut m = monitor();
        let mut source = FakeSource {
            fail_sockets: true,
            transfers: vec![transfer(10, "1.1.1.1:443", 10)],
            ..FakeSource::default()
        };
        m.step(&mut source, 1_000);
        assert_eq!(m.errors, vec!["sockets: access denied".to_string()]);
        assert_eq!(m.snapshot(1_000).apps.len(), 1);
    }

    #[test]
    fn the_interval_since_the_last_step_sets_the_rate() {
        let mut m = monitor();
        let mut source = FakeSource::default();
        m.step(&mut source, 1_000);
        source.transfers = vec![transfer(10, "1.1.1.1:443", 4000)];
        m.step(&mut source, 3_000);
        assert_eq!(m.snapshot(3_000).apps[0].rate.down, 2000, "two seconds passed");
    }
}
