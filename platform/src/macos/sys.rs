// /Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/sys/ioccom.h

// /* copy parameters out */
// #define IOC_OUT         (__uint32_t)0x40000000
// /* copy parameters in */
// #define IOC_IN          (__uint32_t)0x80000000
// /* copy parameters in and out */
// #define IOC_INOUT       (IOC_IN|IOC_OUT)

// #define _IOC(inout, group, num, len) \
// 	(inout | ((len & IOCPARM_MASK) << 16) | ((group) << 8) | (num))
// #define _IOW(g, n, t)     _IOC(IOC_IN,	(g), (n), sizeof(t))
// /* this should be _IORW, but stdio got there first */
// #define _IOWR(g, n, t)    _IOC(IOC_INOUT,	(g), (n), sizeof(t))

// 'i' as u8 = 105
// mem::size_of::<ifreq>() = 32

use core::fmt;
use std::fmt::Debug;

use libc::{c_int, c_ulong, c_void};

// #define SIOCSIFLLADDR   _IOW('i', 60, struct ifreq)     /* set link level addr */
// s = 0x80000000 | 32 << 16 | (105 << 8) | 60
pub(crate) const SIOCSIFLLADDR: c_ulong = 0x8020693c;

// #define SIOCSIFLLADDR   _IORW('i', 158, struct ifreq)     /* set link level addr */
// g = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158
pub(crate) const SIOCGIFLLADDR: c_ulong = 0xc020699e;

pub(crate) trait Sys {
    fn as_sys(&self) -> Box<dyn Sys>;
    fn fmt_as_sys(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int;
    fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int;
    fn close(&self, fd: c_int) -> c_int;
}

impl fmt::Debug for Box<dyn Sys> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_as_sys(f)
    }
}

impl Default for Box<dyn Sys> {
    fn default() -> Box<dyn Sys> {
        Box::new(LibcSys {})
    }
}

impl Clone for Box<dyn Sys> {
    fn clone(&self) -> Self {
        self.as_sys()
    }
}

#[derive(Clone, Debug, Default)]
pub struct LibcSys {}

impl Sys for LibcSys {
    fn as_sys(&self) -> Box<dyn Sys> {
        Box::new(self.clone())
    }

    fn fmt_as_sys(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }

    fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
        unsafe { libc::socket(domain, ty, protocol) }
    }

    fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
        unsafe { libc::ioctl(fd, request, arg) }
    }

    fn close(&self, fd: c_int) -> c_int {
        unsafe { libc::close(fd) }
    }
}

#[cfg(test)]
pub(crate) mod mock {
    use libc::{c_int, c_ulong, c_void};
    use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

    use crate::macos::ifr;

    use super::{Sys, SIOCGIFLLADDR, SIOCSIFLLADDR};

    type KeyValue = RefCell<HashMap<String, String>>;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct MockSys {
        kv: Rc<KeyValue>,
    }

    impl MockSys {
        pub(crate) fn with_nic(self, name: &str, mac_address: &str) -> Self {
            self.kv
                .borrow_mut()
                .insert(name.to_string(), mac_address.to_string());
            self
        }

        pub(crate) fn has_nic(&self, name: &str, expected_mac_address: &str) -> bool {
            match self.kv.borrow().get(name) {
                Some(mac_address) => mac_address == expected_mac_address,
                None => false,
            }
        }
    }

    impl Sys for MockSys {
        fn as_sys(&self) -> Box<dyn Sys> {
            Box::new(self.clone())
        }

        fn fmt_as_sys(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.fmt(f)
        }

        fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            eprintln!("MockSys.socket(domain={domain}, ty={ty}, protocol={protocol})");
            0
        }

        fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            let ifr = ifr::from_c_void_ptr(arg);
            let name = ifr::get_name(ifr);

            match request {
                SIOCGIFLLADDR => {
                    match self.kv.borrow().get(name) {
                        Some(mac_address) => {
                            eprintln!("MockSys.ioctl(fd={fd}, request={request}, {name}) -> {mac_address}");
                            ifr::set_mac_address(ifr, &mac_address);
                            0
                        }
                        _ => -1,
                    }
                }
                SIOCSIFLLADDR => {
                    let mac_address: String;
                    mac_address = ifr::get_mac_address(ifr);
                    eprintln!("MockSys.ioctl(fd={fd}, request={request}, {name}, {mac_address})");
                    self.kv
                        .borrow_mut()
                        .insert(name.to_string(), mac_address.to_string());
                    0
                }
                _ => -1,
            }
        }

        fn close(&self, fd: c_int) -> c_int {
            eprintln!("MockSys.close(fd={fd})");
            0
        }
    }
}
