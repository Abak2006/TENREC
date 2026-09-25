// TENREC - Native Windows Socket & Connection Telemetry
use crate::network::types::{ConnectionInfo, ListeningPortInfo};
use chrono::Utc;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use windows_sys::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable,
};

#[repr(C)]
#[derive(Copy, Clone)]
struct MibTcpRowOwnerPid {
    dw_state: u32,
    dw_local_addr: u32,
    dw_local_port: u32,
    dw_remote_addr: u32,
    dw_remote_port: u32,
    dw_owning_pid: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct MibUdpRowOwnerPid {
    dw_local_addr: u32,
    dw_local_port: u32,
    dw_owning_pid: u32,
}

const AF_INET: u32 = 2;
const TCP_TABLE_OWNER_PID_ALL: u32 = 5;
const UDP_TABLE_OWNER_PID: u32 = 1;

fn tcp_state_to_str(state: u32) -> &'static str {
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

pub fn collect_active_tcp_connections(
    process_names: &HashMap<u32, (String, String)>,
) -> (Vec<ConnectionInfo>, Vec<ListeningPortInfo>) {
    let mut connections = Vec::new();
    let mut listening = Vec::new();
    let now = Utc::now().to_rfc3339();

    unsafe {
        let mut buffer_size: u32 = 0;
        // First call to determine buffer size
        let res = GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut buffer_size,
            1, // sort = true
            AF_INET,
            TCP_TABLE_OWNER_PID_ALL as i32,
            0,
        );

        if res == ERROR_INSUFFICIENT_BUFFER as u32 || buffer_size > 0 {
            let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
            let success = GetExtendedTcpTable(
                buffer.as_mut_ptr() as *mut _,
                &mut buffer_size,
                1,
                AF_INET,
                TCP_TABLE_OWNER_PID_ALL as i32,
                0,
            );

            if success == 0 {
                let num_entries = *(buffer.as_ptr() as *const u32);
                let rows_ptr = buffer.as_ptr().add(4) as *const MibTcpRowOwnerPid;

                for i in 0..num_entries {
                    let row = *rows_ptr.add(i as usize);
                    let local_ip = Ipv4Addr::from(row.dw_local_addr.to_ne_bytes());
                    let remote_ip = Ipv4Addr::from(row.dw_remote_addr.to_ne_bytes());

                    // Network byte order conversion for port numbers
                    let local_port = u16::from_be((row.dw_local_port & 0xFFFF) as u16);
                    let remote_port = u16::from_be((row.dw_remote_port & 0xFFFF) as u16);
                    let state_str = tcp_state_to_str(row.dw_state);

                    let (proc_name, app_name) = process_names
                        .get(&row.dw_owning_pid)
                        .cloned()
                        .unwrap_or_else(|| {
                            (format!("PID:{}", row.dw_owning_pid), "Unknown App".to_string())
                        });

                    if state_str == "LISTEN" {
                        listening.push(ListeningPortInfo {
                            port: local_port,
                            protocol: "TCP".to_string(),
                            pid: row.dw_owning_pid,
                            process_name: proc_name,
                            application_name: app_name,
                            bind_address: local_ip.to_string(),
                            state: state_str.to_string(),
                            is_known_service: local_port < 1024,
                        });
                    } else if row.dw_state == 5 /* ESTABLISHED */ || row.dw_state == 8 /* CLOSE_WAIT */ {
                        connections.push(ConnectionInfo {
                            id: format!("tcp_{}_{}_{}_{}", row.dw_owning_pid, local_port, remote_ip, remote_port),
                            pid: row.dw_owning_pid,
                            process_name: proc_name,
                            application_name: app_name,
                            protocol: "TCP".to_string(),
                            local_address: local_ip.to_string(),
                            local_port,
                            remote_address: remote_ip.to_string(),
                            remote_port,
                            state: state_str.to_string(),
                            hostname: String::new(),
                            bytes_sent: 0,
                            bytes_received: 0,
                            first_seen: now.clone(),
                            last_seen: now.clone(),
                            is_unusual: false,
                            is_new: false,
                        });
                    }
                }
            }
        }
    }

    (connections, listening)
}

pub fn collect_active_udp_endpoints(
    process_names: &HashMap<u32, (String, String)>,
) -> Vec<ListeningPortInfo> {
    let mut endpoints = Vec::new();

    unsafe {
        let mut buffer_size: u32 = 0;
        let res = GetExtendedUdpTable(
            std::ptr::null_mut(),
            &mut buffer_size,
            1,
            AF_INET,
            UDP_TABLE_OWNER_PID as i32,
            0,
        );

        if res == ERROR_INSUFFICIENT_BUFFER as u32 || buffer_size > 0 {
            let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
            let success = GetExtendedUdpTable(
                buffer.as_mut_ptr() as *mut _,
                &mut buffer_size,
                1,
                AF_INET,
                UDP_TABLE_OWNER_PID as i32,
                0,
            );

            if success == 0 {
                let num_entries = *(buffer.as_ptr() as *const u32);
                let rows_ptr = buffer.as_ptr().add(4) as *const MibUdpRowOwnerPid;

                for i in 0..num_entries {
                    let row = *rows_ptr.add(i as usize);
                    let local_ip = Ipv4Addr::from(row.dw_local_addr.to_ne_bytes());
                    let local_port = u16::from_be((row.dw_local_port & 0xFFFF) as u16);

                    let (proc_name, app_name) = process_names
                        .get(&row.dw_owning_pid)
                        .cloned()
                        .unwrap_or_else(|| {
                            (format!("PID:{}", row.dw_owning_pid), "Unknown App".to_string())
                        });

                    endpoints.push(ListeningPortInfo {
                        port: local_port,
                        protocol: "UDP".to_string(),
                        pid: row.dw_owning_pid,
                        process_name: proc_name,
                        application_name: app_name,
                        bind_address: local_ip.to_string(),
                        state: "OPEN".to_string(),
                        is_known_service: local_port < 1024,
                    });
                }
            }
        }
    }

    endpoints
}
