use libc::c_ulong;

#[cfg(not(test))]
use libc::{c_int, c_void};

use super::ioccom;

const IFREQ_SIZE: c_ulong = 32;

// Get link level addr
// SIOCGIFLLADDR = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158 = 0xc020699e
// https://github.com/apple/darwin-xnu/blob/2ff845c2e033bd0ff64b5b6aa6063a1f8f65aa32/bsd/sys/sockio.h#L265
pub(crate) const SIOCGIFLLADDR: c_ulong = ioccom::iorw(ioccom::I, 158, IFREQ_SIZE);

// Set link level addr
// SIOCSIFLLADDR = 0x80000000 | 32 << 16 | (105 << 8) | 60 = 0x8020693c
// https://github.com/apple/darwin-xnu/blob/2ff845c2e033bd0ff64b5b6aa6063a1f8f65aa32/bsd/sys/sockio.h#L146
pub(crate) const SIOCSIFLLADDR: c_ulong = ioccom::iow(ioccom::I, 60, IFREQ_SIZE);

#[cfg(not(test))]
#[cfg(not(tarpaulin_include))]
pub(crate) fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    unsafe { libc::socket(domain, ty, protocol) }
}

#[cfg(not(test))]
#[cfg(not(tarpaulin_include))]
pub(crate) fn ioctl(fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
    unsafe { libc::ioctl(fd, request, arg) }
}

#[cfg(not(test))]
#[cfg(not(tarpaulin_include))]
pub(crate) fn close(fd: c_int) -> c_int {
    unsafe { libc::close(fd) }
}

#[cfg(not(test))]
#[cfg(not(tarpaulin_include))]
pub(crate) fn errno() -> c_int {
    unsafe { *libc::__error() }
}

pub(crate) fn strerror(errno: libc::c_int) -> String {
    let ptr = unsafe { libc::strerror(errno) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    c_str.to_bytes().escape_ascii().to_string()
}

#[cfg(test)]
mod tests {
    use libc::c_ulong;

    use crate::Result;

    use super::{strerror, IFREQ_SIZE, SIOCGIFLLADDR, SIOCSIFLLADDR};

    #[test]
    fn test_ifreq_size() -> Result<()> {
        let expected_size: c_ulong = std::mem::size_of::<libc::ifreq>().try_into()?;

        assert_eq!(IFREQ_SIZE, expected_size);

        Ok(())
    }

    #[test]
    fn test_get_link_level_addr() {
        assert_eq!(SIOCGIFLLADDR, 0xc020699e)
    }

    #[test]
    fn test_set_link_level_addr() {
        assert_eq!(SIOCSIFLLADDR, 0x8020693c)
    }

    #[test]
    fn test_sys_strerror() {
        let errno = 1;

        let strerror = strerror(errno);

        assert_eq!(strerror, "Operation not permitted");
    }

    #[test]
    fn test_sys_strerror_undefined_errno() {
        let errno = 0;

        let strerror = strerror(errno);

        assert_eq!(strerror, "Undefined error: 0");
    }

    #[test]
    fn test_sys_strerror_unknown_errno() {
        let errno = -1;

        let strerror = strerror(errno);

        assert_eq!(strerror, "Unknown error: -1");
    }
}
