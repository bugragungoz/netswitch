//! The IP Helper API: open sockets with their processes, and the byte counters of the adapters.

use super::{SysError, SysResult};
use crate::model::{InOut, Socket};
use crate::tables::{parse_tcp, parse_udp, Family};
use std::ffi::c_void;
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetExtendedTcpTable, GetExtendedUdpTable, GetIfTable2, MIB_IF_TABLE2, TCP_TABLE_OWNER_PID_ALL,
    UDP_TABLE_OWNER_PID,
};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};

/// Calls an IP Helper "fill this buffer" function until the buffer is big enough, and returns
/// the bytes. The buffer is 8-byte aligned, as the tables need.
fn fetch(call: &'static str, mut f: impl FnMut(Option<*mut c_void>, *mut u32) -> u32) -> SysResult<Vec<u8>> {
    let mut buffer: Vec<u64> = Vec::new();
    for _ in 0..8 {
        let mut size = (buffer.len() * 8) as u32;
        let ptr = (!buffer.is_empty()).then_some(buffer.as_mut_ptr() as *mut c_void);
        let ret = f(ptr, &mut size);
        if ret == NO_ERROR.0 {
            let len = buffer.len() * 8;
            // SAFETY: the buffer holds `len` initialized bytes.
            let bytes = unsafe { std::slice::from_raw_parts(buffer.as_ptr() as *const u8, len) };
            return Ok(bytes.to_vec());
        }
        if ret != ERROR_INSUFFICIENT_BUFFER.0 {
            return Err(SysError::Code { call, code: ret });
        }
        // Sockets open between the two calls make the table grow; leave some room.
        buffer = vec![0u64; (size as usize).div_ceil(8) + 256];
    }
    Err(SysError::Other(format!("{call} kept asking for a bigger buffer")))
}

/// Every TCP and UDP socket, IPv4 and IPv6, with the process that owns it.
pub fn sockets() -> SysResult<Vec<Socket>> {
    let mut out = Vec::new();
    for (family, af) in [(Family::V4, AF_INET.0 as u32), (Family::V6, AF_INET6.0 as u32)] {
        let tcp = fetch("GetExtendedTcpTable", |ptr, size| unsafe {
            GetExtendedTcpTable(ptr, size, false, af, TCP_TABLE_OWNER_PID_ALL, 0)
        })?;
        out.extend(parse_tcp(&tcp, family));
        let udp = fetch("GetExtendedUdpTable", |ptr, size| unsafe {
            GetExtendedUdpTable(ptr, size, false, af, UDP_TABLE_OWNER_PID, 0)
        })?;
        out.extend(parse_udp(&udp, family));
    }
    Ok(out)
}

/// The byte counters of the physical adapters that are up, summed. Filter drivers and virtual
/// adapters are left out: they see the same bytes again.
pub fn adapter_counters() -> SysResult<InOut> {
    let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    let ret = unsafe { GetIfTable2(&mut table) };
    if ret != NO_ERROR {
        return Err(SysError::Code {
            call: "GetIfTable2",
            code: ret.0,
        });
    }
    let mut total = InOut::default();
    // SAFETY: GetIfTable2 succeeded, so `table` points to NumEntries rows until FreeMibTable.
    unsafe {
        let rows = std::slice::from_raw_parts((*table).Table.as_ptr(), (*table).NumEntries as usize);
        for row in rows {
            let flags = row.InterfaceAndOperStatusFlags._bitfield;
            let hardware = flags & 0b01 != 0;
            let filter = flags & 0b10 != 0;
            if hardware && !filter && row.OperStatus == IfOperStatusUp {
                total.down = total.down.saturating_add(row.InOctets);
                total.up = total.up.saturating_add(row.OutOctets);
            }
        }
        FreeMibTable(table as *const c_void);
    }
    Ok(total)
}
