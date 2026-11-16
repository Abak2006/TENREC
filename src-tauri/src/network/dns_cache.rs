// TENREC - Hostnames from the local Windows DNS client cache
//
// No DNS traffic is generated: names are read from the resolver cache
// (DnsGetCacheDataTable, undocumented but long-stable) and their addresses are looked
// up with DNS_QUERY_NO_WIRE_QUERY, which only consults the local cache.
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, Instant};
use windows_sys::Win32::NetworkManagement::Dns::{
    DnsFree, DnsFreeFlat, DnsFreeRecordList, DnsQuery_W, DNS_QUERY_NO_WIRE_QUERY, DNS_RECORDA, DNS_RECORDW, DNS_TYPE_A, DNS_TYPE_AAAA,
};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

const REFRESH_EVERY: Duration = Duration::from_secs(15);
const MAX_ENTRIES: usize = 8000;

#[repr(C)]
struct CacheEntry {
    next: *mut CacheEntry,
    name: *mut u16,
    kind: u16,
    data_length: u16,
    flags: u32,
}

type GetCacheTableFn = unsafe extern "system" fn(*mut *mut CacheEntry) -> i32;

fn cached_names() -> Vec<String> {
    let mut out = Vec::new();
    unsafe {
        let lib = LoadLibraryA(c"dnsapi.dll".as_ptr() as *const u8);
        if lib.is_null() {
            return out;
        }
        let Some(f) = GetProcAddress(lib, c"DnsGetCacheDataTable".as_ptr() as *const u8) else { return out };
        let f: GetCacheTableFn = std::mem::transmute(f);
        let mut head: *mut CacheEntry = std::ptr::null_mut();
        if f(&mut head) == 0 {
            return out;
        }
        let mut cur = head;
        while !cur.is_null() {
            let e = &*cur;
            if !e.name.is_null() && (e.kind == DNS_TYPE_A || e.kind == DNS_TYPE_AAAA) {
                let mut len = 0;
                while *e.name.add(len) != 0 {
                    len += 1;
                }
                out.push(String::from_utf16_lossy(std::slice::from_raw_parts(e.name, len)));
            }
            let next = e.next;
            if !e.name.is_null() {
                DnsFree(e.name as *const _, DnsFreeFlat);
            }
            DnsFree(cur as *const _, DnsFreeFlat);
            cur = next;
        }
    }
    out.sort();
    out.dedup();
    out
}

fn cached_addresses(name: &str, kind: u16) -> Vec<IpAddr> {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut out = Vec::new();
    unsafe {
        let mut records: *mut DNS_RECORDA = std::ptr::null_mut();
        if DnsQuery_W(wide.as_ptr(), kind, DNS_QUERY_NO_WIRE_QUERY, std::ptr::null_mut(), &mut records, std::ptr::null_mut()) != 0 {
            return out;
        }
        let mut cur = records as *mut DNS_RECORDW;
        while !cur.is_null() {
            let r = &*cur;
            if r.wType == DNS_TYPE_A {
                out.push(IpAddr::V4(Ipv4Addr::from(r.Data.A.IpAddress.to_ne_bytes())));
            } else if r.wType == DNS_TYPE_AAAA {
                out.push(IpAddr::V6(Ipv6Addr::from(r.Data.AAAA.Ip6Address.IP6Byte)));
            }
            cur = r.pNext;
        }
        DnsFree(records as *const _, DnsFreeRecordList);
    }
    out
}

/// IP address -> hostname, accumulated across refreshes (cache entries expire quickly).
pub struct DnsCache {
    map: HashMap<IpAddr, String>,
    last: Option<Instant>,
}

impl Default for DnsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsCache {
    pub fn new() -> Self {
        Self { map: HashMap::new(), last: None }
    }

    pub fn refresh_if_due(&mut self) {
        if self.last.map(|t| t.elapsed() < REFRESH_EVERY).unwrap_or(false) {
            return;
        }
        self.last = Some(Instant::now());
        for name in cached_names() {
            for kind in [DNS_TYPE_A, DNS_TYPE_AAAA] {
                for ip in cached_addresses(&name, kind) {
                    self.map.insert(ip, name.clone());
                }
            }
        }
        if self.map.len() > MAX_ENTRIES {
            self.map.clear();
        }
    }

    pub fn lookup(&self, ip: &str) -> Option<String> {
        ip.parse::<IpAddr>().ok().and_then(|ip| self.map.get(&ip).cloned())
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
