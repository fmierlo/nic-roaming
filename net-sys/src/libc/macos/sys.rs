use libc::{c_int, c_ulong, c_void};
use std::{fmt::Debug, ops::Deref};

mod ioccom {
    use libc::c_ulong;

    // /Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/sys/ioccom.h

    // Ioctl's have the command encoded in the lower word, and the size of
    // any in or out parameters in the upper word.  The high 3 bits of the
    // upper word are used to encode the in/out status of the parameter.

    // param char 'i' as c_ulong
    pub(super) const I: c_ulong = 105;
    // parameter length, at most 13 bits
    const IOCPARM_MASK: c_ulong = 0x1fff;
    // copy parameters out
    const IOC_OUT: c_ulong = 0x40000000;
    // copy parameters in
    const IOC_IN: c_ulong = 0x80000000;
    // copy parameters in and out
    const IOC_INOUT: c_ulong = IOC_IN | IOC_OUT;

    #[cfg(not(tarpaulin_include))]
    const fn ioc(inout: c_ulong, group: c_ulong, num: c_ulong, len: c_ulong) -> c_ulong {
        inout | ((len & IOCPARM_MASK) << 16) | ((group) << 8) | (num)
    }

    #[cfg(not(tarpaulin_include))]
    pub(super) const fn iow(group: c_ulong, num: c_ulong, len: c_ulong) -> c_ulong {
        ioc(IOC_IN, group, num, len)
    }

    #[cfg(not(tarpaulin_include))]
    pub(super) const fn iorw(group: c_ulong, num: c_ulong, len: c_ulong) -> c_ulong {
        ioc(IOC_INOUT, group, num, len)
    }
}

const IFREQ_SIZE: c_ulong = 32;

// Get link level addr
// SIOCGIFLLADDR = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158 = 0xc020699e
// https://github.com/apple/darwin-xnu/blob/2ff845c2e033bd0ff64b5b6aa6063a1f8f65aa32/bsd/sys/sockio.h#L265
pub(super) const SIOCGIFLLADDR: c_ulong = ioccom::iorw(ioccom::I, 158, IFREQ_SIZE);

// Set link level addr
// SIOCSIFLLADDR = 0x80000000 | 32 << 16 | (105 << 8) | 60 = 0x8020693c
// https://github.com/apple/darwin-xnu/blob/2ff845c2e033bd0ff64b5b6aa6063a1f8f65aa32/bsd/sys/sockio.h#L146
pub(super) const SIOCSIFLLADDR: c_ulong = ioccom::iow(ioccom::I, 60, IFREQ_SIZE);

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
struct LibcSys {}

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
    use libc::c_ulong;

    #[test]
    fn test_ifreq_size() {
        let expected_size: c_ulong = std::mem::size_of::<libc::ifreq>().try_into().unwrap();

        assert_eq!(super::IFREQ_SIZE, expected_size);
    }

    #[test]
    fn test_get_link_level_addr() {
        assert_eq!(super::SIOCGIFLLADDR, 0xc020699e)
    }

    #[test]
    fn test_set_link_level_addr() {
        assert_eq!(super::SIOCSIFLLADDR, 0x8020693c)
    }

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
    fn test_sys_box_default() {
        let expected_default = "BoxSys(LibcSys)";

        let box_sys = super::BoxSys::default();

        assert_eq!(format!("{:?}", box_sys), expected_default);
    }

    #[test]
    fn test_sys_box_debug() {
        let sys = super::mock::MockSys::default();
        let expected_debug = "BoxSys(MockSys { then_errno: RefCell { value: None }, mock: RefCell { value: MockStore { .. } } })";

        let box_sys = super::BoxSys(Box::new(sys));

        assert_eq!(format!("{:?}", box_sys), expected_debug);
    }

    #[test]
    fn test_sys_box_deref() {
        let sys = super::mock::MockSys::default();
        let expected_deref = "MockSys { then_errno: RefCell { value: None }, mock: RefCell { value: MockStore { .. } } }";

        let deref_box_sys = &*super::BoxSys(Box::new(sys));

        assert_eq!(format!("{:?}", deref_box_sys), expected_deref);
    }
}

#[cfg(test)]
pub(super) mod mock {
    use super::super::{ifname::IfName, ifreq};
    use super::Sys;
    use crate::LinkLevelAddress;
    use libc::{c_int, c_ulong, c_void};
    use std::clone::Clone;
    use std::{any::Any, cell::RefCell, cmp::PartialEq, fmt::Debug, rc::Rc};

    #[derive(Default)]
    pub struct Mock {
        store: Vec<Box<dyn Any>>,
    }

    impl Debug for Mock {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MockStore").finish_non_exhaustive()
        }
    }

    impl Mock {
        pub fn on<T: Any + Clone>(&mut self, value: T) {
            self.store.insert(0, Box::new(value));
        }

        pub fn get<T: Any + Clone>(&self) -> Result<T, &str> {
            for item in self.store.iter() {
                match item.downcast_ref::<T>() {
                    Some(value) => return Ok(value.clone()),
                    None => (),
                };
            }
            Err("Not found")
        }
    }

    #[derive(Clone, Copy)]
    pub(crate) struct Socket(
        pub(crate) (c_int, c_int, c_int),
        pub(crate) (Option<c_int>, Option<c_int>),
    );

    #[derive(Clone, Copy)]
    pub(crate) struct IoCtl(
        pub(crate) (c_int, c_ulong, IfName, Option<LinkLevelAddress>),
        pub(crate) (Option<c_int>, Option<LinkLevelAddress>),
    );

    #[derive(Clone, Copy)]
    pub(crate) struct Close(pub(crate) (c_int,), pub(crate) (Option<c_int>,));

    #[derive(Clone, Debug, Default)]
    pub(crate) struct MockSys {
        then_errno: RefCell<Option<c_int>>,
        mock: Rc<RefCell<Mock>>,
    }

    impl MockSys {
        pub(crate) fn on<T: Any + Clone>(self, value: T) -> Self {
            (*self.mock).borrow_mut().on(value);
            self
        }

        fn handle_errno(&self, errno: &Option<c_int>) -> c_int {
            match errno {
                Some(errno) => {
                    *self.then_errno.borrow_mut() = Some(*errno);
                    -1
                }
                None => {
                    *self.then_errno.borrow_mut() = None;
                    0
                }
            }
        }
    }

    fn matches<'a, T: Debug + PartialEq, U>(lhs: T, rhs: T, ret: U) -> U {
        if lhs != rhs {
            panic!("Error: {:?} not match {:?}", rhs, lhs)
        }
        ret
    }

    impl Sys for MockSys {
        fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            let socket_args = (domain, ty, protocol);
            let mock = (*self.mock).borrow();
            let (fd, errno) = match mock.get::<Socket>() {
                Ok(Socket(args, ret)) => matches(args, socket_args, ret),
                Err(err) => panic!("Error: {}", err),
            };

            let ret = match fd {
                Some(fd) => fd,
                None => return self.handle_errno(&errno),
            };
            eprintln!("MockSys.socket(domain={domain}, ty={ty}, protocol={protocol}) -> (ret={ret}, errno={errno:?})");
            ret
        }

        fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            let ifreq = ifreq::from_mut_ptr(arg);
            let ifname = ifreq::get_name(ifreq);
            let lladdr_in = ifreq::get_lladdr(ifreq);

            let lladdr_in = if lladdr_in != "00:00:00:00:00:00".parse().unwrap() {
                Some(lladdr_in)
            } else {
                None
            };

            let ioctl_args = (fd, request, ifname, lladdr_in);

            let mock = (*self.mock).borrow();
            let (errno, lladdr_out) = match mock.get::<IoCtl>() {
                Ok(IoCtl(args, ret)) => matches(args, ioctl_args, ret),
                Err(err) => panic!("Error: {}", err),
            };

            if let Some(lladdr) = lladdr_out {
                ifreq::set_lladdr(ifreq, &lladdr);
            }

            let ret = self.handle_errno(&errno);
            eprintln!("MockSys.ioctl(fd={fd}, request={request}, ifname={ifname:?}, lladdr={lladdr_in:?}) -> (ret={ret}, errno={errno:?}, lladdr={lladdr_out:?})");
            return ret;
        }

        fn close(&self, fd: c_int) -> c_int {
            let close_args = (fd,);

            let mock = (*self.mock).borrow();
            let (errno,) = match mock.get::<Close>() {
                Ok(Close(args, ret)) => matches(args, close_args, ret),
                Err(err) => panic!("Error: {}", err),
            };

            let ret = self.handle_errno(&errno);
            eprintln!("MockSys.close(fd={fd}) -> (ret={ret}, errno={errno:?})");
            return ret;
        }

        fn errno(&self) -> c_int {
            (*self.then_errno.borrow()).unwrap()
        }
    }
}
