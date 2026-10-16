//! Reads the system on Windows. Everything here needs Windows; most of it needs administrator
//! rights, which the app asks for when it starts.

pub mod firewall;
pub mod iphlp;
pub mod process;
pub mod reverse;
pub mod source;
pub mod traffic;

#[derive(Debug, thiserror::Error)]
pub enum SysError {
    #[error(transparent)]
    Windows(#[from] ::windows::core::Error),
    #[error("{call} failed with error {code}")]
    Code { call: &'static str, code: u32 },
    #[error("the traffic trace could not start: {0}")]
    Trace(String),
    #[error("{0}")]
    Other(String),
}

pub type SysResult<T> = Result<T, SysError>;
