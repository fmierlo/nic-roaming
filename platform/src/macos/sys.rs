use std::{fmt::Debug, ops::Deref};

use libc::{c_int, c_ulong, c_void};

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

// #define SIOCSIFLLADDR   _IOW('i', 60, struct ifreq)     /* set link level addr */
// s = 0x80000000 | 32 << 16 | (105 << 8) | 60
pub(crate) const SIOCSIFLLADDR: c_ulong = 0x8020693c;

// #define SIOCSIFLLADDR   _IORW('i', 158, struct ifreq)     /* set link level addr */
// g = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158
pub(crate) const SIOCGIFLLADDR: c_ulong = 0xc020699e;

pub(crate) trait Sys: Debug {
    fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int;
    fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int;
    fn close(&self, fd: c_int) -> c_int;
}

#[derive(Debug, Default)]
pub(crate) struct BoxSys(pub(crate) Box<dyn Sys>);

impl Default for Box<dyn Sys> {
    fn default() -> Self {
        Box::new(LibcSys::default())
    }
}

impl Deref for BoxSys {
    type Target = Box<dyn Sys>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
pub(crate) struct LibcSys {}

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
pub(crate) mod mock {
    use libc::{c_int, c_ulong, c_void};
    use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

    use crate::macos::ifreq::{self};

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
        fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            eprintln!("MockSys.socket(domain={domain}, ty={ty}, protocol={protocol})");
            0
        }

        fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            let ifreq = ifreq::from_mut_ptr(arg);
            let name = match ifreq::get_name(ifreq) {
                Ok(name) => name,
                Err(err) => {
                    eprintln!(
                        "ERROR: MockSys.ioctl(fd={fd}, request={request}, name=none) -> err={err}"
                    );
                    return -1;
                }
            };

            match request {
                SIOCGIFLLADDR => match self.kv.borrow().get(&name) {
                    Some(mac_address) => {
                        eprintln!("MockSys.ioctl(fd={fd}, request=SIOCGIFLLADDR, name={name}) -> mac_address={mac_address}");
                        match ifreq::set_mac_address(ifreq, &mac_address) {
                            Ok(_) => 0,
                            Err(err) => {
                                eprintln!("ERROR: MockSys.ioctl(fd={fd}, request=SIOCGIFLLADDR, name={name}) -> err={err}");
                                -1
                            }
                        }
                    }
                    None => {
                        eprintln!("ERROR: MockSys.ioctl(fd={fd}, request=SIOCGIFLLADDR, name={name}) -> mac_address=none");
                        -1
                    }
                },
                SIOCSIFLLADDR => match ifreq::get_mac_address(ifreq) {
                    Ok(mac_address) => {
                        eprintln!("MockSys.ioctl(fd={fd}, request=SIOCSIFLLADDR, name={name}, mac_address={mac_address}) -> true");
                        self.kv.borrow_mut().insert(name, mac_address);
                        0
                    }
                    Err(err) => {
                        eprintln!("ERROR: MockSys.ioctl(fd={fd}, request=SIOCSIFLLADDR, name={name}, mac_address=none) -> err={err}");
                        -1
                    }
                },
                request => {
                    eprintln!("ERROR: MockSys.ioctl(fd={fd}, request={request}, name={name}) -> err='Invalid request value'");
                    -1
                }
            }
        }

        fn close(&self, fd: c_int) -> c_int {
            eprintln!("MockSys.close(fd={fd})");
            0
        }
    }
}
