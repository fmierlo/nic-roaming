use libc::{c_int, c_ulong, c_void};
use std::rc::Rc;

// #define SIOCSIFLLADDR   _IOW('i', 60, struct ifreq)     /* set link level addr */
// s = 0x80000000 | 32 << 16 | (105 << 8) | 60
pub(crate) const SIOCSIFLLADDR: c_ulong = 0x8020693c;

// #define SIOCSIFLLADDR   _IORW('i', 158, struct ifreq)     /* set link level addr */
// g = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158
pub(crate) const SIOCGIFLLADDR: c_ulong = 0xc020699e;

pub fn new() -> Rc<LibcSys> {
    LibcSys::new()
}

pub trait Sys {
    fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int;
    fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int;
    fn close(&self, fd: c_int) -> c_int;
}

pub struct LibcSys {}

impl LibcSys {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {})
    }
}

impl Sys for LibcSys {
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
pub(crate) mod tests {
    use libc::{c_int, c_ulong, c_void};
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::macos::ifr;

    use super::{Sys, SIOCGIFLLADDR, SIOCSIFLLADDR};

    pub(crate) struct MockSys {
        nic_list: RefCell<HashMap<String, String>>,
    }

    impl MockSys {
        pub(crate) fn new() -> Rc<Self> {
            Rc::new(Self {
                nic_list: RefCell::new(HashMap::new()),
            })
        }

        pub(crate) fn with_nic(self: Rc<Self>, name: &str, mac_address: &str) -> Rc<Self> {
            self.set_nic(name, mac_address);
            self
        }

        pub(crate) fn set_nic(&self, name: &str, mac_address: &str) {
            self.nic_list
                .borrow_mut()
                .insert(name.to_string(), mac_address.to_string());
        }

        pub(crate) fn has_nic(&self, name: &str, expected_mac_address: &str) -> bool {
            match self.nic_list.borrow().get(name) {
                Some(mac_address) => mac_address == expected_mac_address,
                None => false,
            }
        }
    }

    impl Sys for MockSys {
        fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            println!("socket: domain={domain} ty={ty} protocol={protocol}");
            0
        }

        fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            println!("ioctl: fd={fd} request={request} arg={arg:?}");

            let ifr = ifr::from_c_void_ptr(arg);
            let name = ifr::get_name(ifr);

            match request {
                SIOCGIFLLADDR => {
                    match self.nic_list.borrow().get(name) {
                        Some(mac_address) => ifr::set_mac_address(ifr, &mac_address),
                        _ => {}
                    };
                }
                SIOCSIFLLADDR => {
                    let mac_address: String;
                    mac_address = ifr::get_mac_address(ifr);
                    self.set_nic(name, &mac_address);
                }
                _ => {}
            }

            0
        }

        fn close(&self, fd: c_int) -> c_int {
            println!("close: fd={fd}");
            0
        }
    }
}
