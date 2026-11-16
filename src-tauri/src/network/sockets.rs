// TENREC - Socket table collection (IPv4 + IPv6) via IP Helper
use crate::network::types::{ConnectionInfo, ListeningPortInfo};
use crate::util::{is_public_ip, is_wildcard_bind};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
use windows_sys::Win32::NetworkManagement::IpHelper::{GetExtendedTcpTable, GetExtendedUdpTable};

const AF_INET: u32 = 2;
const AF_INET6: u32 = 23;
const TCP_TABLE_OWNER_PID_ALL: i32 = 5;
const UDP_TABLE_OWNER_PID: i32 = 1;

#[repr(C)]
#[derive(Copy, Clone)]
struct TcpRow4 {
    state: u32,
    local_addr: u32,
    local_port: u32,
    remote_addr: u32,
    remote_port: u32,
    pid: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct TcpRow6 {
    local_addr: [u8; 16],
    local_scope: u32,
    local_port: u32,
    remote_addr: [u8; 16],
    remote_scope: u32,
    remote_port: u32,
    state: u32,
    pid: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct UdpRow4 {
    local_addr: u32,
    local_port: u32,
    pid: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct UdpRow6 {
    local_addr: [u8; 16],
    local_scope: u32,
    local_port: u32,
    pid: u32,
}

/// Process name / application name lookup by PID.
pub type ProcessNames = HashMap<u32, (String, String)>;

fn port(raw: u32) -> u16 {
    u16::from_be((raw & 0xFFFF) as u16)
}

fn tcp_state(state: u32) -> &'static str {
    match state {
        1 => "CLOSED",
        2 => "LISTEN",
        3 => "SYN_SENT",
        4 => "SYN_RCVD",
        5 => "ESTABLISHED",
        6 => "FIN_WAIT1",
        7 => "FIN_WAIT2",
        8 => "CLOSE_WAIT",
        9 => "CLOSING",
        10 => "LAST_ACK",
        11 => "TIME_WAIT",
        12 => "DELETE_TCB",
        _ => "UNKNOWN",
    }
}

/// Calls an IP Helper table function, growing the buffer when the table changes
/// size between the size query and the real call.
fn fetch_table(call: impl Fn(*mut core::ffi::c_void, *mut u32) -> u32) -> Option<Vec<u8>> {
    let mut size: u32 = 0;
    let first = call(std::ptr::null_mut(), &mut size);
    if first != ERROR_INSUFFICIENT_BUFFER && first != NO_ERROR {
        return None;
    }
    for _ in 0..4 {
        // Over-allocate a little: the table can grow between calls.
        let mut buf = vec![0u8; size as usize + 1024];
        let mut len = buf.len() as u32;
        let rc = call(buf.as_mut_ptr() as *mut _, &mut len);
        if rc == NO_ERROR {
            return Some(buf);
        }
        if rc != ERROR_INSUFFICIENT_BUFFER {
            return None;
        }
        size = len;
    }
    None
}

/// Reads rows of type T that follow the leading u32 entry count.
fn rows<T: Copy>(buf: &[u8]) -> Vec<T> {
    if buf.len() < 4 {
        return Vec::new();
    }
    let count = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    let row_size = std::mem::size_of::<T>();
    let max = (buf.len() - 4) / row_size;
    (0..count.min(max))
        .map(|i| unsafe { std::ptr::read_unaligned(buf.as_ptr().add(4 + i * row_size) as *const T) })
        .collect()
}

fn names(map: &ProcessNames, pid: u32) -> (String, String) {
    map.get(&pid).cloned().unwrap_or_else(|| {
        if pid == 4 {
            ("System".to_string(), "Windows kernel".to_string())
        } else {
            (format!("PID {pid}"), "Unknown (access denied or exited)".to_string())
        }
    })
}

struct RawTcp {
    state: u32,
    local: IpAddr,
    local_port: u16,
    remote: IpAddr,
    remote_port: u16,
    pid: u32,
}

fn tcp_rows() -> Vec<RawTcp> {
    let mut out = Vec::new();
    if let Some(buf) = fetch_table(|p, s| unsafe { GetExtendedTcpTable(p, s, 0, AF_INET, TCP_TABLE_OWNER_PID_ALL, 0) }) {
        for r in rows::<TcpRow4>(&buf) {
            out.push(RawTcp {
                state: r.state,
                local: IpAddr::V4(Ipv4Addr::from(r.local_addr.to_ne_bytes())),
                local_port: port(r.local_port),
                remote: IpAddr::V4(Ipv4Addr::from(r.remote_addr.to_ne_bytes())),
                remote_port: port(r.remote_port),
                pid: r.pid,
            });
        }
    }
    if let Some(buf) = fetch_table(|p, s| unsafe { GetExtendedTcpTable(p, s, 0, AF_INET6, TCP_TABLE_OWNER_PID_ALL, 0) }) {
        for r in rows::<TcpRow6>(&buf) {
            out.push(RawTcp {
                state: r.state,
                local: IpAddr::V6(Ipv6Addr::from(r.local_addr)),
                local_port: port(r.local_port),
                remote: IpAddr::V6(Ipv6Addr::from(r.remote_addr)),
                remote_port: port(r.remote_port),
                pid: r.pid,
            });
        }
    }
    out
}

/// Collects live TCP connections and TCP/UDP listening endpoints.
pub fn collect_sockets(process_names: &ProcessNames) -> (Vec<ConnectionInfo>, Vec<ListeningPortInfo>) {
    let mut connections = Vec::new();
    let mut listening = Vec::new();

    for r in tcp_rows() {
        let (pname, aname) = names(process_names, r.pid);
        let ip_version = if r.local.is_ipv4() { 4 } else { 6 };
        match r.state {
            2 => listening.push(ListeningPortInfo {
                port: r.local_port,
                protocol: "TCP".to_string(),
                ip_version,
                pid: r.pid,
                process_name: pname,
                application_name: aname,
                all_interfaces: is_wildcard_bind(&r.local.to_string()),
                bind_address: r.local.to_string(),
            }),
            // ESTABLISHED, SYN_SENT, CLOSE_WAIT: sockets that represent a live peer.
            3 | 5 | 8 => connections.push(ConnectionInfo {
                id: format!("tcp{}_{}_{}_{}_{}", ip_version, r.pid, r.local_port, r.remote, r.remote_port),
                pid: r.pid,
                process_name: pname,
                application_name: aname,
                protocol: "TCP".to_string(),
                ip_version,
                local_address: r.local.to_string(),
                local_port: r.local_port,
                remote_address: r.remote.to_string(),
                remote_port: r.remote_port,
                state: tcp_state(r.state).to_string(),
                is_public_remote: is_public_ip(&r.remote),
                is_new: false,
            }),
            _ => {}
        }
    }

    if let Some(buf) = fetch_table(|p, s| unsafe { GetExtendedUdpTable(p, s, 0, AF_INET, UDP_TABLE_OWNER_PID, 0) }) {
        for r in rows::<UdpRow4>(&buf) {
            let (pname, aname) = names(process_names, r.pid);
            let addr = Ipv4Addr::from(r.local_addr.to_ne_bytes()).to_string();
            listening.push(ListeningPortInfo {
                port: port(r.local_port),
                protocol: "UDP".to_string(),
                ip_version: 4,
                pid: r.pid,
                process_name: pname,
                application_name: aname,
                all_interfaces: is_wildcard_bind(&addr),
                bind_address: addr,
            });
        }
    }
    if let Some(buf) = fetch_table(|p, s| unsafe { GetExtendedUdpTable(p, s, 0, AF_INET6, UDP_TABLE_OWNER_PID, 0) }) {
        for r in rows::<UdpRow6>(&buf) {
            let (pname, aname) = names(process_names, r.pid);
            let addr = Ipv6Addr::from(r.local_addr).to_string();
            listening.push(ListeningPortInfo {
                port: port(r.local_port),
                protocol: "UDP".to_string(),
                ip_version: 6,
                pid: r.pid,
                process_name: pname,
                application_name: aname,
                all_interfaces: is_wildcard_bind(&addr),
                bind_address: addr,
            });
        }
    }

    listening.sort_by(|a, b| a.port.cmp(&b.port).then(a.protocol.cmp(&b.protocol)));
    listening.dedup_by(|a, b| a.port == b.port && a.protocol == b.protocol && a.bind_address == b.bind_address && a.pid == b.pid);
    (connections, listening)
}

/// PID -> number of live TCP connections, used to annotate the process list.
pub fn connection_counts(connections: &[ConnectionInfo]) -> HashMap<u32, usize> {
    let mut m = HashMap::new();
    for c in connections {
        *m.entry(c.pid).or_insert(0) += 1;
    }
    m
}
