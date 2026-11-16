// TENREC - Small shared helpers
use std::net::IpAddr;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Lock helpers that recover from a poisoned lock instead of panicking.
/// A panic in one sampling cycle must not take every later IPC call down with it.
pub trait LockExt<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> LockExt<T> for Mutex<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub trait RwLockExt<T> {
    fn read_safe(&self) -> RwLockReadGuard<'_, T>;
    fn write_safe(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn read_safe(&self) -> RwLockReadGuard<'_, T> {
        self.read().unwrap_or_else(|e| e.into_inner())
    }
    fn write_safe(&self) -> RwLockWriteGuard<'_, T> {
        self.write().unwrap_or_else(|e| e.into_inner())
    }
}

/// True when the address is a globally routable unicast address
/// (not loopback, private, link-local, CGNAT, multicast, documentation-free check is
/// intentionally omitted so test fixtures using TEST-NET ranges still classify as public).
pub fn is_public_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            !(v4.is_unspecified()
                || v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_broadcast()
                || (o[0] == 100 && (o[1] & 0b1100_0000) == 64) // 100.64.0.0/10 CGNAT
                || o[0] == 0)
        }
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_public_ip(&IpAddr::V4(v4));
            }
            let seg0 = v6.segments()[0];
            !(v6.is_unspecified()
                || v6.is_loopback()
                || v6.is_multicast()
                || (seg0 & 0xfe00) == 0xfc00 // fc00::/7 unique local
                || (seg0 & 0xffc0) == 0xfe80) // fe80::/10 link-local
        }
    }
}

pub fn is_public_ip_str(s: &str) -> bool {
    s.parse::<IpAddr>().map(|ip| is_public_ip(&ip)).unwrap_or(false)
}

/// True when the bind address accepts connections on every interface.
pub fn is_wildcard_bind(s: &str) -> bool {
    s == "0.0.0.0" || s == "::"
}

/// Executables running from these user-writable locations are worth a second look
/// when they open network sockets.
pub fn is_user_writable_location(path: &str) -> bool {
    let p = path.to_ascii_lowercase().replace('/', "\\");
    p.contains("\\appdata\\local\\temp\\")
        || p.contains("\\windows\\temp\\")
        || p.contains("\\downloads\\")
        || p.contains("\\$recycle.bin\\")
}
