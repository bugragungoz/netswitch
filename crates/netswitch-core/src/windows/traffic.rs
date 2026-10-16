//! Traffic per process from Event Tracing for Windows, without a capture driver.
//!
//! Microsoft-Windows-Kernel-Network reports every TCP and UDP send and receive with the process,
//! the size and both addresses. Microsoft-Windows-DNS-Client reports the names programs look up
//! and the addresses they get, so a connection can show "youtube.com" instead of a bare address.
//! Both need administrator rights.
//!
//! The callbacks run on the trace's own thread. They only add to counters keyed by process and
//! address pair; the monitor takes the counters once a second, so a busy download costs a map
//! update per event, not an allocation.

use super::{SysError, SysResult};
use crate::events::{
    dns_name, dns_result_addresses, event_port, kernel_network_transfer, DNS_CLIENT, DNS_QUERY_COMPLETED,
    KERNEL_NETWORK, KERNEL_NETWORK_KEYWORDS,
};
use crate::model::{Direction, Protocol, Transfer};
use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::trace::{stop_trace_by_name, UserTrace};
use ferrisetw::{EventRecord, SchemaLocator};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};

/// The trace session's name, as `logman query -ets` lists it.
pub const SESSION_NAME: &str = "netSwitch traffic";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Key {
    pid: u32,
    protocol: Protocol,
    direction: Direction,
    local: SocketAddr,
    remote: SocketAddr,
}

type Counters = Arc<Mutex<HashMap<Key, u64>>>;
type Names = Arc<Mutex<Vec<(IpAddr, String)>>>;

pub struct TrafficSource {
    _trace: UserTrace,
    counters: Counters,
    names: Names,
}

fn on_kernel_network(counters: &Counters, record: &EventRecord, locator: &SchemaLocator) {
    let Some((protocol, direction)) = kernel_network_transfer(record.event_id()) else {
        return;
    };
    let Ok(schema) = locator.event_schema(record) else {
        return;
    };
    let parser = Parser::create(record, &schema);
    let (Ok(pid), Ok(size), Ok(daddr), Ok(saddr), Ok(dport), Ok(sport)) = (
        parser.try_parse::<u32>("PID"),
        parser.try_parse::<u32>("size"),
        parser.try_parse::<IpAddr>("daddr"),
        parser.try_parse::<IpAddr>("saddr"),
        parser.try_parse::<u16>("dport"),
        parser.try_parse::<u16>("sport"),
    ) else {
        return;
    };
    // The events give addresses as the connection sees them, for sends and receives alike: the
    // source is this computer's end and the destination the other end.
    let key = Key {
        pid,
        protocol,
        direction,
        local: SocketAddr::new(saddr, event_port(sport)),
        remote: SocketAddr::new(daddr, event_port(dport)),
    };
    if let Ok(mut map) = counters.lock() {
        *map.entry(key).or_insert(0) += u64::from(size);
    }
}

fn on_dns_client(names: &Names, record: &EventRecord, locator: &SchemaLocator) {
    if record.event_id() != DNS_QUERY_COMPLETED {
        return;
    }
    let Ok(schema) = locator.event_schema(record) else {
        return;
    };
    let parser = Parser::create(record, &schema);
    let (Ok(name), Ok(results)) = (
        parser.try_parse::<String>("QueryName"),
        parser.try_parse::<String>("QueryResults"),
    ) else {
        return;
    };
    if parser.try_parse::<u32>("QueryStatus").is_ok_and(|status| status != 0) {
        return;
    }
    let Some(name) = dns_name(&name) else { return };
    let found = dns_result_addresses(&results);
    if found.is_empty() {
        return;
    }
    if let Ok(mut list) = names.lock() {
        list.extend(found.into_iter().map(|ip| (ip, name.clone())));
    }
}

impl TrafficSource {
    /// Starts the trace. Fails without administrator rights.
    pub fn start() -> SysResult<Self> {
        // A session left running by a netSwitch that did not close cleanly holds the name.
        let _ = stop_trace_by_name(SESSION_NAME);

        let counters: Counters = Arc::default();
        let names: Names = Arc::default();

        let net_counters = counters.clone();
        let kernel_network = Provider::by_guid(KERNEL_NETWORK)
            .any(KERNEL_NETWORK_KEYWORDS)
            .add_callback(move |record: &EventRecord, locator: &SchemaLocator| {
                on_kernel_network(&net_counters, record, locator)
            })
            .build();

        let dns_names = names.clone();
        let dns_client = Provider::by_guid(DNS_CLIENT)
            .add_callback(move |record: &EventRecord, locator: &SchemaLocator| {
                on_dns_client(&dns_names, record, locator)
            })
            .build();

        let trace = UserTrace::new()
            .named(SESSION_NAME.to_string())
            .enable(kernel_network)
            .enable(dns_client)
            .start_and_process()
            .map_err(|e| SysError::Trace(format!("{e:?}")))?;
        Ok(Self {
            _trace: trace,
            counters,
            names,
        })
    }

    /// The transfers counted since the last call.
    pub fn take_transfers(&self) -> Vec<Transfer> {
        let map = match self.counters.lock() {
            Ok(mut map) => std::mem::take(&mut *map),
            Err(_) => return Vec::new(),
        };
        map.into_iter()
            .map(|(key, bytes)| Transfer {
                pid: key.pid,
                protocol: key.protocol,
                direction: key.direction,
                bytes,
                local: key.local,
                remote: key.remote,
            })
            .collect()
    }

    /// Addresses and the names programs looked them up by, since the last call.
    pub fn take_names(&self) -> Vec<(IpAddr, String)> {
        self.names
            .lock()
            .map(|mut list| std::mem::take(&mut *list))
            .unwrap_or_default()
    }
}
