use libc::{c_int, c_ulong, c_void};
use std::{fmt::Debug, ops::Deref};

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
pub(super) const SIOCSIFLLADDR: c_ulong = 0x8020693c;

// #define SIOCSIFLLADDR   _IORW('i', 158, struct ifreq)     /* set link level addr */
// g = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158
pub(super) const SIOCGIFLLADDR: c_ulong = 0xc020699e;

pub(super) fn strerror(errno: c_int) -> String {
    let ptr = unsafe { libc::strerror(errno) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    c_str.to_bytes().escape_ascii().to_string()
}

pub(super) trait Sys: Debug {
    fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int;
    fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int;
    fn close(&self, fd: c_int) -> c_int;
    fn errno(&self) -> c_int;
}

#[derive(Debug, Default)]
pub(super) struct BoxSys(pub(super) Box<dyn Sys>);

#[cfg(not(tarpaulin_include))]
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
pub(super) struct LibcSys {}

#[cfg(not(tarpaulin_include))]
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

    fn errno(&self) -> c_int {
        unsafe { *libc::__error() }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_sys_strerror() {
        let errno = 1;

        let strerror = super::strerror(errno);

        assert_eq!(strerror, "Operation not permitted");
    }

    #[test]
    fn test_sys_strerror_undefined_errno() {
        let errno = 0;

        let strerror = super::strerror(errno);

        assert_eq!(strerror, "Undefined error: 0");
    }

    #[test]
    fn test_sys_strerror_unknown_errno() {
        let errno = -1;

        let strerror = super::strerror(errno);

        assert_eq!(strerror, "Unknown error: -1");
    }

    #[test]
    fn test_sys_boxsys_deref() {
        let sys = super::mock::MockSys::default().with_nic(
            "enx".try_into().unwrap(),
            "01:02:03:04:05:06".parse().unwrap(),
        );

        let dyn_sys: &Box<dyn super::Sys> = &*super::BoxSys(Box::new(sys.clone()));

        assert_eq!(format!("{:?}", dyn_sys), format!("{:?}", sys));
    }
}

#[cfg(test)]
pub(super) mod mock {
    use super::super::{ifname::IfName, ifreq};
    use super::{Sys, SIOCGIFLLADDR, SIOCSIFLLADDR};
    use crate::LinkLevelAddress;
    use libc::{c_int, c_ulong, c_void};
    use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

    type KeyValue = RefCell<HashMap<IfName, LinkLevelAddress>>;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct MockSys {
        kv: Rc<KeyValue>,
    }

    impl MockSys {
        pub(crate) fn with_nic(self, ifname: IfName, lladdr: LinkLevelAddress) -> Self {
            self.kv.borrow_mut().insert(ifname, lladdr);
            self
        }

        pub(crate) fn has_nic(&self, ifname: &IfName, expected_lladdr: &LinkLevelAddress) -> bool {
            match self.kv.borrow().get(ifname) {
                Some(lladdr) => lladdr == expected_lladdr,
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
            let ifname = ifreq::get_name(ifreq);

            match request {
                SIOCGIFLLADDR => match self.kv.borrow().get(&ifname) {
                    Some(lladdr) => {
                        eprintln!("MockSys.ioctl(fd={fd}, request=SIOCGIFLLADDR, ifname={ifname}) -> lladd={lladdr}");
                        ifreq::set_lladdr(ifreq, lladdr);
                        0
                    }
                    None => {
                        eprintln!("ERROR: MockSys.ioctl(fd={fd}, request=SIOCGIFLLADDR, ifname={ifname}) -> lladd=none");
                        -1
                    }
                },
                SIOCSIFLLADDR => {
                    let lladdr = ifreq::get_lladdr(ifreq);
                    eprintln!("MockSys.ioctl(fd={fd}, request=SIOCSIFLLADDR, ifname={ifname}, lladd={lladdr}) -> true");
                    self.kv.borrow_mut().insert(ifname, lladdr);
                    0
                }
                request => {
                    eprintln!("ERROR: MockSys.ioctl(fd={fd}, request={request}, ifname={ifname}) -> err='Invalid request value'");
                    -1
                }
            }
        }

        fn close(&self, fd: c_int) -> c_int {
            eprintln!("MockSys.close(fd={fd})");
            0
        }

        fn errno(&self) -> c_int {
            libc::EPERM // Operation not permitted
        }
    }
}
