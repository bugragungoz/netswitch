//! The monitor thread: one step a second, the latest state kept for the window to read.

use crate::settings;
use netswitch_core::blocklist::BlockedApp;
use netswitch_core::model::{InOut, Snapshot};
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::AppHandle;

/// What the window shows, as of the last step.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    /// False on systems other than Windows, where netSwitch measures nothing.
    pub supported: bool,
    pub admin: bool,
    /// Why traffic per app is not measured, when it is not.
    pub trace_error: Option<String>,
    /// Parts that failed in the last step.
    pub errors: Vec<String>,
    pub snapshot: Snapshot,
    pub blocked: Vec<BlockedApp>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            supported: cfg!(windows),
            admin: false,
            trace_error: None,
            errors: Vec::new(),
            snapshot: Snapshot {
                at: 0,
                per_app: false,
                rate: InOut::default(),
                total: InOut::default(),
                history: Vec::new(),
                apps: Vec::new(),
                connections: Vec::new(),
            },
            blocked: Vec::new(),
        }
    }
}

pub struct Shared {
    state: Mutex<State>,
    settings: Mutex<settings::Store>,
    /// Set after a firewall change, so the next step reads the rules again.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub refresh_blocklist: AtomicBool,
}

/// A poisoned lock only means a panic elsewhere; the data is still the last good state.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Shared {
    pub fn new(settings: settings::Store) -> Self {
        Self {
            state: Mutex::new(State::default()),
            settings: Mutex::new(settings),
            refresh_blocklist: AtomicBool::new(false),
        }
    }

    pub fn state(&self) -> MutexGuard<'_, State> {
        lock(&self.state)
    }

    pub fn settings(&self) -> MutexGuard<'_, settings::Store> {
        lock(&self.settings)
    }
}

/// An app that reached the internet for the first time in this run, as the window is told.
#[cfg(windows)]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOnline {
    pub key: String,
    pub name: String,
}

#[cfg(windows)]
pub fn start(app: AppHandle, shared: Arc<Shared>) {
    std::thread::Builder::new()
        .name("netswitch-monitor".into())
        .spawn(move || windows::run(app, shared))
        .expect("monitor thread");
}

#[cfg(not(windows))]
pub fn start(_app: AppHandle, _shared: Arc<Shared>) {
    // Nothing to measure: the window says netSwitch runs on Windows.
}

#[cfg(windows)]
mod windows {
    use super::{AppOnline, Shared};
    use netswitch_core::monitor::{Monitor, Source};
    use netswitch_core::windows::{process, reverse, source::WindowsSource};
    use std::net::IpAddr;
    use std::sync::atomic::Ordering;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    use tauri::{AppHandle, Emitter};

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Names found by reverse DNS, waiting for the next step.
    type Found = Arc<Mutex<Vec<(IpAddr, String)>>>;

    /// Looks up names one at a time on its own thread: a lookup can take seconds.
    fn reverse_dns() -> (mpsc::Sender<IpAddr>, Found) {
        let (tx, rx) = mpsc::channel::<IpAddr>();
        let found = Arc::new(Mutex::new(Vec::new()));
        let out = found.clone();
        let _ = std::thread::Builder::new()
            .name("netswitch-reverse-dns".into())
            .spawn(move || {
                for ip in rx {
                    if let Some(name) = reverse::lookup(ip) {
                        if let Ok(mut list) = out.lock() {
                            list.push((ip, name));
                        }
                    }
                }
            });
        (tx, found)
    }

    pub fn run(app: AppHandle, shared: Arc<Shared>) {
        let mut source = WindowsSource::start();
        {
            let mut state = shared.state();
            state.admin = process::is_elevated();
            state.trace_error = source.trace_error.clone();
        }
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        let mut monitor = Monitor::new(source.per_app(), &system_root, process::program_path, |path: &str| {
            process::file_description(path)
        });
        let (lookups, found) = reverse_dns();

        loop {
            let started = Instant::now();
            if shared.refresh_blocklist.swap(false, Ordering::SeqCst) {
                monitor.refresh_blocklist();
            }
            if let Ok(mut list) = found.lock() {
                for (ip, name) in list.drain(..) {
                    monitor.set_host(ip, name);
                }
            }

            let now = now_ms();
            let online = monitor.step(&mut source, now);
            for ip in monitor.take_unnamed() {
                if !(ip.is_loopback() || ip.is_unspecified() || ip.is_multicast()) {
                    let _ = lookups.send(ip);
                }
            }
            let snapshot = monitor.snapshot(now);
            {
                let mut state = shared.state();
                state.snapshot = snapshot;
                state.blocked = monitor.blocklist().apps.clone();
                state.errors = monitor.errors.clone();
            }
            if !online.is_empty() {
                let apps: Vec<AppOnline> = online
                    .into_iter()
                    .map(|a| AppOnline {
                        key: a.key,
                        name: a.name,
                    })
                    .collect();
                let _ = app.emit("apps-online", apps);
            }

            std::thread::sleep(Duration::from_millis(1000).saturating_sub(started.elapsed()));
        }
    }
}
