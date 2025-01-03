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
        let expected_debug = "BoxSys(MockSys { .. })";

        let box_sys = super::BoxSys(Box::new(sys));

        assert_eq!(format!("{:?}", box_sys), expected_debug);
    }

    #[test]
    fn test_sys_box_deref() {
        let sys = super::mock::MockSys::default();
        let expected_deref = "MockSys { .. }";

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
    use std::any::type_name;
    use std::clone::Clone;
    use std::fmt::Debug;
    use std::ops::Deref;
    use std::{any::Any, cell::RefCell, cmp::PartialEq, rc::Rc};

    #[derive(Default)]
    pub(crate) struct Mock {
        store: RefCell<Vec<(Box<dyn Any>, &'static str)>>,
    }

    impl Mock {
        fn on<T: Any + Clone>(&self, value: T) {
            self.store
                .borrow_mut()
                .insert(0, (Box::new(value), type_name::<T>()));
        }

        pub fn next<T: Any + Clone>(&self) -> T {
            let (next, next_type_name) = match self.store.borrow_mut().pop() {
                Some(next) => next,
                None => panic!(
                    "{:?}: type not found, predicate list is empty",
                    type_name::<T>()
                ),
            };

            match next.downcast::<T>() {
                Ok(next) => *next,
                Err(_) => panic!(
                    "{:?}: type not compatible with next value type {:?}",
                    type_name::<T>(),
                    next_type_name
                ),
            }
        }

        pub fn assert_next<T, V, U, P>(&self, destructure: P) -> U
        where
            P: Fn(&T) -> (V, (&V, &U)),
            T: Any + Clone,
            V: Clone + PartialEq + Debug,
            U: Clone,
        {
            let next = self.next();

            let (lhs, (rhs, ret)) = destructure(&next);

            if &lhs == rhs {
                ret.clone()
            } else {
                panic!(
                    "{:?}: type value {:?} don't match value {:?}",
                    type_name::<T>(),
                    lhs,
                    rhs
                )
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Socket(pub(crate) (c_int, c_int, c_int), pub(crate) (c_int,));

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct IoCtl(
        pub(crate) (c_int, c_ulong, IfName, Option<LinkLevelAddress>),
        pub(crate) (c_int, Option<LinkLevelAddress>),
    );

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Close(pub(crate) (c_int,), pub(crate) (c_int,));

    #[derive(Clone, Copy, Debug)]
    pub(crate) struct ErrNo(pub(crate) (), pub(crate) (c_int,));

    #[derive(Clone, Default)]
    pub(crate) struct MockSys {
        mock: Rc<Mock>,
    }

    impl Deref for MockSys {
        type Target = Rc<Mock>;

        fn deref(&self) -> &Self::Target {
            &self.mock
        }
    }

    impl Debug for MockSys {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MockSys").finish_non_exhaustive()
        }
    }

    impl MockSys {
        pub(crate) fn on<T: Any + Clone>(self, value: T) -> Self {
            self.mock.on(value);
            self
        }
    }

    impl Sys for MockSys {
        fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            let socket_args = (domain, ty, protocol);
            let (ret,) = self.assert_next(|Socket(args, ret)| (socket_args, (args, ret)));
            eprintln!("MockSys.socket(domain={domain}, ty={ty}, protocol={protocol}) -> ret={ret}");
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
            let (ret, lladdr_out) = self.assert_next(|IoCtl(args, ret)| (ioctl_args, (args, ret)));

            if let Some(lladdr) = lladdr_out {
                ifreq::set_lladdr(ifreq, &lladdr);
            }

            eprintln!("MockSys.ioctl(fd={fd}, request={request}, ifname={ifname:?}, lladdr={lladdr_in:?}) -> (ret={ret}, lladdr={lladdr_out:?})");
            ret
        }

        fn close(&self, fd: c_int) -> c_int {
            let close_args = (fd,);
            let (ret,) = self.assert_next(|Close(args, ret)| (close_args, (args, ret)));
            eprintln!("MockSys.close(fd={fd}) -> ret={ret}");
            ret
        }

        fn errno(&self) -> c_int {
            let errno_args = ();
            let (ret,) = self.assert_next(|ErrNo(args, ret)| (errno_args, (args, ret)));
            eprintln!("MockSys.errno() -> ret={ret}");
            ret
        }
    }
}
