//! The monitor's view of a Windows system.

use super::{firewall, iphlp, traffic::TrafficSource};
use crate::blocklist::FirewallRule;
use crate::model::{InOut, Socket, Transfer};
use crate::monitor::Source;
use std::net::IpAddr;

pub struct WindowsSource {
    traffic: Option<TrafficSource>,
    /// Why traffic per app is not measured, when it is not.
    pub trace_error: Option<String>,
}

impl WindowsSource {
    /// Starts the traffic trace. Without administrator rights it cannot start; the source then
    /// reports sockets and adapter totals only.
    pub fn start() -> Self {
        match TrafficSource::start() {
            Ok(traffic) => Self {
                traffic: Some(traffic),
                trace_error: None,
            },
            Err(e) => Self {
                traffic: None,
                trace_error: Some(e.to_string()),
            },
        }
    }
}

impl Source for WindowsSource {
    fn take_transfers(&mut self) -> Vec<Transfer> {
        self.traffic
            .as_ref()
            .map(TrafficSource::take_transfers)
            .unwrap_or_default()
    }

    fn take_names(&mut self) -> Vec<(IpAddr, String)> {
        self.traffic.as_ref().map(TrafficSource::take_names).unwrap_or_default()
    }

    fn sockets(&mut self) -> Result<Vec<Socket>, String> {
        iphlp::sockets().map_err(|e| e.to_string())
    }

    fn adapters(&mut self) -> Result<InOut, String> {
        iphlp::adapter_counters().map_err(|e| e.to_string())
    }

    fn firewall_rules(&mut self) -> Result<Vec<FirewallRule>, String> {
        firewall::read_rules().map_err(|e| e.to_string())
    }

    fn per_app(&self) -> bool {
        self.traffic.is_some()
    }
}
