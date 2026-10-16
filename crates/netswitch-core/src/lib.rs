//! netSwitch's core: what it counts and how, independent of the window.
//!
//! The pure modules build and test on any system. The
//! `windows` module reads the system: traffic per process from Event Tracing for Windows,
//! sockets from the IP Helper tables, adapter counters, program names, and the firewall.

pub mod aggregate;
pub mod apps;
pub mod blocklist;
pub mod events;
pub mod model;
pub mod monitor;
pub mod net;
pub mod rules;
pub mod tables;

#[cfg(windows)]
pub mod windows;
