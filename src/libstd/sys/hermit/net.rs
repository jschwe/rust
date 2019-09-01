#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::fmt;
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{SocketAddr, Shutdown, Ipv4Addr, Ipv6Addr};
use crate::time::{self,Duration};
use crate::sys::{unsupported, Void};
use crate::convert::TryFrom;
use crate::{ptr,str};
use crate::sys::hermit::thread::Tid;
use crate::ffi::c_void;

#[allow(unused_extern_crates)]
pub extern crate smoltcp;

use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::socket::SocketSet;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};

extern "C" {
    fn sys_spawn(id: *mut Tid, func: extern "C" fn(usize), arg: usize, prio: u8, core_id: isize) -> i32;
    fn sys_network_init(sem: *const c_void, ip: &mut [u8; 4], gateway: &mut [u8; 4], mac: &mut [u8; 18]) -> i32;
    fn sys_sem_init(sem: *mut *const c_void, value: u32) -> i32;
    fn sys_sem_timedwait(sem: *const c_void, ms: u32) -> i32;
    fn sys_is_polling() -> bool;
    fn sys_yield();
}

const MAX_MSG_SIZE: usize = 1792;

extern "C" fn networkd(_: usize) {
    let mut ip: [u8; 4] = [0; 4];
    let mut gateway: [u8; 4] = [0; 4];
    let mut mac: [u8; 18] = [0; 18];
    let mut sem: *const c_void = ptr::null();
    
    let ret = unsafe { sys_sem_init(&mut sem as *mut *const c_void, 0) };
    if ret != 0 {
        return;
    }

    let ret = unsafe { sys_network_init(sem, &mut ip, &mut gateway, &mut mac) };
    if ret != 0 {
        return;
    }

    let mut neighbor_cache_entries = [None; 8];
    let mut neighbor_cache = NeighborCache::new(&mut neighbor_cache_entries[..]);
    let mac_str = str::from_utf8(&mac).unwrap();
    let ethernet_addr = EthernetAddress([
        u8::from_str_radix(&mac_str[0..2], 16).unwrap(),
        u8::from_str_radix(&mac_str[3..5], 16).unwrap(),
        u8::from_str_radix(&mac_str[6..8], 16).unwrap(),
        u8::from_str_radix(&mac_str[9..11], 16).unwrap(),
        u8::from_str_radix(&mac_str[12..14], 16).unwrap(),
        u8::from_str_radix(&mac_str[15..17], 16).unwrap(),
    ]);
    let mut ip_addrs = [IpCidr::new(IpAddress::v4(ip[0], ip[1], ip[2], ip[3]), 24)];
    let default_gw = Ipv4Address::new(gateway[0], gateway[1], gateway[2], gateway[3]);
    let mut routes_storage = [None; 1];
    let mut routes = Routes::new(&mut routes_storage[..]);
    routes.add_default_ipv4_route(default_gw).unwrap();
    let device = DeviceNet::new();

    let mut iface = EthernetInterfaceBuilder::new(device)
        .ethernet_addr(ethernet_addr)
        .neighbor_cache(neighbor_cache)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .finalize();

    let mut socket_set_entries: [_; 2] = Default::default();
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let start = time::SystemTime::now();

    loop {
        let timestamp = time::SystemTime::now().duration_since(start).unwrap();
        let timestamp_ms = (timestamp.as_secs() * 1_000) as i64 + (timestamp.subsec_nanos() / 1_000_000) as i64;

        match iface.poll(&mut socket_set, smoltcp::time::Instant::from_millis(timestamp_ms)) {
            Ok(_) => {},
            Err(_) => {}
        }

        if unsafe{ !sys_is_polling() } {
            let delay = match iface.poll_delay(&socket_set, smoltcp::time::Instant::from_millis(timestamp_ms)) {
                  Some(duration) => {
                      // Calculate the maximum sleep time in milliseconds.
                      if duration.millis() > 0 {
                          duration.millis()
                      } else {
                          1
                      }
                  },
                  None => { 1 },
            };

            unsafe {
               let _ = sys_sem_timedwait(sem, delay as u32);
            }
        }
    }
}

#[derive(Debug)]
pub struct DeviceNet {
    mtu: usize,
}

impl DeviceNet {
    /// Creates a network device for HermitCore.
    ///
    /// Every packet transmitted through this device will be received through it
    /// in FIFO order.
    pub fn new() -> DeviceNet {
        DeviceNet { mtu: 1500 }
    }
}

impl<'a> Device<'a> for DeviceNet {
    type RxToken = RxToken;
    type TxToken = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.max_transmission_unit = self.mtu;
    	cap
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        extern "C" {
           fn sys_netread(buf: usize, len: usize) -> usize;
        }

        let mut rx = RxToken::new();
        let ret = unsafe { sys_netread(rx.buffer.as_mut_ptr() as usize, MAX_MSG_SIZE) };

        if ret > 0 {
            let tx = TxToken::new();
            rx.resize(ret);

            Some((rx, tx))
        } else {
            None
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(TxToken::new())
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: [u8; MAX_MSG_SIZE],
    len: usize,
}

impl RxToken {
    pub fn new() -> RxToken {
        RxToken {
            buffer: [0; MAX_MSG_SIZE],
            len: MAX_MSG_SIZE,
        }
    }

    pub fn resize(&mut self, len: usize) {
        if len <= MAX_MSG_SIZE {
            self.len = len;
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: smoltcp::time::Instant, f: F) -> smoltcp::Result<R>
    where
    	F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let (first, _) = self.buffer.split_at_mut(self.len);
        f(first)
    }
}

#[doc(hidden)]
pub struct TxToken;

impl TxToken {
    pub const fn new() -> Self {
    	TxToken {}
    }

    fn write(&self, data: usize, len: usize) -> usize {
        extern "C" {
           fn sys_netwrite(buf: usize, len: usize) -> usize;
        }

        unsafe { sys_netwrite(data, len) }
    }
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, _timestamp: smoltcp::time::Instant, len: usize, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        if result.is_ok() {
            self.write(buffer.as_ptr() as usize, len);
        }
        result
    }
}

// Iinitializes HermitCore's network stack
pub unsafe fn init() -> io::Result<()> {
    // create thread to handle IP packets
    let mut tid: Tid = 0;
    let ret = sys_spawn(&mut tid as *mut Tid,
                        networkd /* thread entry point */,
                        0 /* no argument */,
                        3 /* = priority above normal */,
                        0 /* networkd should always use core 0 */
              );

    if ret != 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "Unable to create thread"));
    }

    // make sure that the thread gets computation time
    sys_yield();

    Ok(())
}

pub struct TcpStream(Void);

impl TcpStream {
    pub fn connect(_: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        unsupported()
    }

    pub fn connect_timeout(_: &SocketAddr, _: Duration) -> io::Result<TcpStream> {
        unsupported()
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        match self.0 {}
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        match self.0 {}
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        match self.0 {}
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        match self.0 {}
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn read(&self, _: &mut [u8]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn read_vectored(&self, _: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn write(&self, _: &[u8]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn write_vectored(&self, _: &[IoSlice<'_>]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self.0 {}
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        match self.0 {}
    }

    pub fn shutdown(&self, _: Shutdown) -> io::Result<()> {
        match self.0 {}
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        match self.0 {}
    }

    pub fn set_nodelay(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        match self.0 {}
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        match self.0 {}
    }

    pub fn ttl(&self) -> io::Result<u32> {
        match self.0 {}
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        match self.0 {}
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {}
    }
}

pub struct TcpListener(Void);

impl TcpListener {
    pub fn bind(_: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        unsupported()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        match self.0 {}
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        match self.0 {}
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        match self.0 {}
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        match self.0 {}
    }

    pub fn ttl(&self) -> io::Result<u32> {
        match self.0 {}
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        match self.0 {}
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        match self.0 {}
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {}
    }
}

pub struct UdpSocket(Void);

impl UdpSocket {
    pub fn bind(_: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        unsupported()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self.0 {}
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        match self.0 {}
    }

    pub fn recv_from(&self, _: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        match self.0 {}
    }

    pub fn peek_from(&self, _: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        match self.0 {}
    }

    pub fn send_to(&self, _: &[u8], _: &SocketAddr) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        match self.0 {}
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        match self.0 {}
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        match self.0 {}
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        match self.0 {}
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        match self.0 {}
    }

    pub fn set_broadcast(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        match self.0 {}
    }

    pub fn set_multicast_loop_v4(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        match self.0 {}
    }

    pub fn set_multicast_ttl_v4(&self, _: u32) -> io::Result<()> {
        match self.0 {}
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        match self.0 {}
    }

    pub fn set_multicast_loop_v6(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        match self.0 {}
    }

    pub fn join_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr)
                         -> io::Result<()> {
        match self.0 {}
    }

    pub fn join_multicast_v6(&self, _: &Ipv6Addr, _: u32)
                         -> io::Result<()> {
        match self.0 {}
    }

    pub fn leave_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr)
                          -> io::Result<()> {
        match self.0 {}
    }

    pub fn leave_multicast_v6(&self, _: &Ipv6Addr, _: u32)
                          -> io::Result<()> {
        match self.0 {}
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        match self.0 {}
    }

    pub fn ttl(&self) -> io::Result<u32> {
        match self.0 {}
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        match self.0 {}
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        match self.0 {}
    }

    pub fn recv(&self, _: &mut [u8]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn send(&self, _: &[u8]) -> io::Result<usize> {
        match self.0 {}
    }

    pub fn connect(&self, _: io::Result<&SocketAddr>) -> io::Result<()> {
        match self.0 {}
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {}
    }
}

pub struct LookupHost(Void);

impl LookupHost {
    pub fn port(&self) -> u16 {
        match self.0 {}
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        match self.0 {}
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: &str) -> io::Result<LookupHost> {
        unsupported()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: (&'a str, u16)) -> io::Result<LookupHost> {
        unsupported()
    }
}

#[allow(nonstandard_style)]
pub mod netc {
    pub const AF_INET: u8 = 0;
    pub const AF_INET6: u8 = 1;
    pub type sa_family_t = u8;

    #[derive(Copy, Clone)]
    pub struct in_addr {
        pub s_addr: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in {
        pub sin_family: sa_family_t,
        pub sin_port: u16,
        pub sin_addr: in_addr,
    }

    #[derive(Copy, Clone)]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in6 {
        pub sin6_family: sa_family_t,
        pub sin6_port: u16,
        pub sin6_addr: in6_addr,
        pub sin6_flowinfo: u32,
        pub sin6_scope_id: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr {
    }

    pub type socklen_t = usize;
}
