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
        let sys = super::LibcSys::default();
        let expected_debug = "BoxSys(LibcSys)";

        let box_sys = super::BoxSys(Box::new(sys));

        assert_eq!(format!("{:?}", box_sys), expected_debug);
    }

    #[test]
    fn test_sys_box_deref() {
        let sys = super::LibcSys::default();
        let expected_deref = "LibcSys";

        let deref_box_sys = &*super::BoxSys(Box::new(sys));

        assert_eq!(format!("{:?}", deref_box_sys), expected_deref);
    }
}

#[cfg(test)]
pub(super) mod mock {
    use super::Sys;
    use libc::{c_int, c_ulong, c_void};
    use mockdown::{Mockdown, ExpectStore};
    use std::fmt::Debug;

    #[derive(Debug, PartialEq)]
    pub(crate) struct Socket(pub libc::c_int, pub libc::c_int, pub libc::c_int);
    #[derive(Debug, PartialEq)]
    pub(crate) struct IoCtl(pub (libc::c_int, libc::c_ulong), pub *mut libc::c_void);
    #[derive(Debug, PartialEq)]
    pub(crate) struct Close(pub libc::c_int);
    #[derive(Debug)]
    pub(crate) struct ErrNo();

    #[derive(Clone, Default)]
    pub(crate) struct MockSys(ExpectStore);

    impl Debug for MockSys {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("MockSys").finish()
        }
    }

    impl Mockdown for MockSys {
        fn store(&self) -> &ExpectStore {
            &self.0
        }
    }

    impl Sys for MockSys {
        fn socket(&self, domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            let args = Socket(domain, ty, protocol);
            self.on_mock(args).unwrap()
        }

        fn ioctl(&self, fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            let args = IoCtl((fd, request), arg);
            self.on_mock(args).unwrap()
        }

        fn close(&self, fd: c_int) -> c_int {
            let args = Close(fd);
            self.on_mock(args).unwrap()
        }

        fn errno(&self) -> c_int {
            let args = ErrNo();
            self.on_mock(args).unwrap()
        }
    }
}
