use libc::c_int;

#[cfg(not(test))]
use libc::{c_ulong, c_void, size_t, ssize_t};

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
pub(crate) fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    unsafe { libc::read(fd, buf, count) }
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

pub(crate) fn strerror(errno: c_int) -> String {
    let ptr = unsafe { libc::strerror(errno) };
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    c_str.to_bytes().escape_ascii().to_string()
}

#[cfg(test)]
mod tests {
    use super::strerror;

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
