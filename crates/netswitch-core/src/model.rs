//! What the monitor sees and what it reports to the window.

use serde::Serialize;
use std::net::{IpAddr, SocketAddr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Received: download.
    In,
    /// Sent: upload.
    Out,
}

/// Bytes one process sent or received on one address pair, as the traffic source reports them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub pid: u32,
    pub protocol: Protocol,
    pub direction: Direction,
    pub bytes: u64,
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    DeleteTcb,
}

impl TcpState {
    /// The MIB_TCP_STATE value of the IP Helper tables.
    pub fn from_mib(value: u32) -> Option<Self> {
        Some(match value {
            1 => Self::Closed,
            2 => Self::Listen,
            3 => Self::SynSent,
            4 => Self::SynReceived,
            5 => Self::Established,
            6 => Self::FinWait1,
            7 => Self::FinWait2,
            8 => Self::CloseWait,
            9 => Self::Closing,
            10 => Self::LastAck,
            11 => Self::TimeWait,
            12 => Self::DeleteTcb,
            _ => return None,
        })
    }
}

/// One row of the TCP or UDP tables: a socket and the process that owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Socket {
    pub pid: u32,
    pub protocol: Protocol,
    pub local: SocketAddr,
    /// TCP only; UDP sockets have no fixed remote end.
    pub remote: Option<SocketAddr>,
    pub state: Option<TcpState>,
}

/// The program behind a process. Processes of the same program are one app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppIdentity {
    /// Lowercased full path of the program, or `pid:<n>` when the path cannot be read.
    pub key: String,
    /// The file description of the program ("Google Chrome"), else its file name without ".exe".
    pub name: String,
    pub path: Option<String>,
}

/// The file name of a program without ".exe": the name of an app without a description.
pub fn program_stem(path: &str) -> &str {
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    file.strip_suffix(".exe")
        .or_else(|| file.strip_suffix(".EXE"))
        .unwrap_or(file)
}

impl AppIdentity {
    pub fn from_path(path: &str, description: Option<String>) -> Self {
        let name = description
            .filter(|d| !d.trim().is_empty())
            .map(|d| d.trim().to_string())
            .unwrap_or_else(|| program_stem(path).to_string());
        Self {
            key: path.to_lowercase(),
            name,
            path: Some(path.to_string()),
        }
    }

    /// True while the name is only the file name, with no description read yet.
    pub fn has_default_name(&self) -> bool {
        self.path.as_deref().is_none_or(|p| program_stem(p) == self.name)
    }

    /// A process whose program cannot be read: the System process, or one that already ended.
    pub fn unknown(pid: u32) -> Self {
        let name = match pid {
            0 => "System Idle Process".to_string(),
            4 => "System".to_string(),
            _ => format!("Process {pid}"),
        };
        Self {
            key: format!("pid:{pid}"),
            name,
            path: None,
        }
    }
}

/// A pair of byte counts: received (download) and sent (upload). A rate is bytes per second.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct InOut {
    #[serde(rename = "in")]
    pub down: u64,
    #[serde(rename = "out")]
    pub up: u64,
}

impl InOut {
    pub fn new(down: u64, up: u64) -> Self {
        Self { down, up }
    }

    pub fn add(&mut self, direction: Direction, bytes: u64) {
        match direction {
            Direction::In => self.down = self.down.saturating_add(bytes),
            Direction::Out => self.up = self.up.saturating_add(bytes),
        }
    }

    pub fn sum(self) -> u64 {
        self.down.saturating_add(self.up)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStat {
    pub key: String,
    pub name: String,
    pub path: Option<String>,
    pub pids: Vec<u32>,
    pub rate: InOut,
    pub total: InOut,
    /// Bytes per second over the last minute, oldest first.
    pub history: Vec<InOut>,
    pub connections: u32,
    pub listening: u32,
    pub hosts: u32,
    /// Milliseconds since the Unix epoch.
    pub first_seen: u64,
    pub last_active: Option<u64>,
    pub blocked: bool,
    /// Part of Windows: netSwitch does not block it.
    pub protected: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStat {
    pub app: String,
    pub app_name: String,
    pub protocol: Protocol,
    pub local_port: u16,
    pub remote_ip: IpAddr,
    pub remote_port: u16,
    /// The name the app asked DNS for, else the reverse DNS name, when known.
    pub host: Option<String>,
    /// The well-known service of the remote port, such as "https".
    pub service: Option<&'static str>,
    /// True for addresses on this computer or the local network.
    pub local_network: bool,
    pub state: Option<TcpState>,
    pub open: bool,
    pub rate: InOut,
    pub total: InOut,
    pub last_active: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub at: u64,
    /// Whether traffic per app is measured. Without it only the totals of the network adapters are.
    pub per_app: bool,
    pub rate: InOut,
    pub total: InOut,
    /// Bytes per second of all adapters over the last five minutes, oldest first.
    pub history: Vec<InOut>,
    pub apps: Vec<AppStat>,
    pub connections: Vec<ConnectionStat>,
}
