use std::fmt::{Debug, Display};

use libc::{c_char, c_int, c_void, ssize_t};

use crate::ifname::IfName;
use crate::lladdr::LinkLevelAddress;
use crate::Result;

use super::defs::sio;
use super::types::ifreq::{IfReq, IfReqAsPtr};

#[cfg(not(test))]
use super::sys;
#[cfg(test)]
use mocks::sys;

#[derive(Clone, PartialEq, Eq)]
enum Error {
    OpenLocalDgram(c_int, c_int),
    GetLinkLevelAddress(c_int, IfName, c_int, c_int),
    SetLinkLevelAddress(c_int, IfName, LinkLevelAddress, c_int, c_int),
    Read(c_int, ssize_t, c_int),
    Close(c_int, c_int, c_int),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OpenLocalDgram(ret, errno) => f
                .debug_struct("Socket::OpenLocalDgramError")
                .field("ret", ret)
                .field("errno", errno)
                .field("strerror", &sys::strerror(*errno))
                .finish(),
            Error::GetLinkLevelAddress(fd, ifname, ret, errno) => f
                .debug_struct("Socket::GetLinkLevelAddressError")
                .field("fd", fd)
                .field("ifname", ifname)
                .field("ret", ret)
                .field("errno", errno)
                .field("strerror", &sys::strerror(*errno))
                .finish(),
            Error::SetLinkLevelAddress(fd, ifname, lladdr, ret, errno) => f
                .debug_struct("Socket::SetLinkLevelAddressError")
                .field("fd", fd)
                .field("ifname", ifname)
                .field("lladdr", lladdr)
                .field("ret", ret)
                .field("errno", errno)
                .field("strerror", &sys::strerror(*errno))
                .finish(),
            Error::Read(fd, ret, errno) => f
                .debug_struct("Socket::Read")
                .field("fd", fd)
                .field("ret", ret)
                .field("errno", errno)
                .field("strerror", &sys::strerror(*errno))
                .finish(),
            Error::Close(fd, ret, errno) => f
                .debug_struct("Socket::CloseError")
                .field("fd", fd)
                .field("ret", ret)
                .field("errno", errno)
                .field("strerror", &sys::strerror(*errno))
                .finish(),
        }
    }
}

pub(crate) fn open_local_dgram() -> Result<OpenSocket> {
    match sys::socket(libc::PF_LOCAL, libc::SOCK_DGRAM, 0) {
        fd if fd >= 0 => Ok(OpenSocket { fd }),
        ret => {
            let errno = sys::errno();
            Err(Error::OpenLocalDgram(ret, errno).into())
        }
    }
}

pub(crate) fn open_route_raw() -> Result<OpenSocket> {
    match sys::socket(libc::PF_ROUTE, libc::SOCK_RAW, 0) {
        fd if fd >= 0 => Ok(OpenSocket { fd }),
        ret => {
            let errno = sys::errno();
            Err(Error::OpenLocalDgram(ret, errno).into())
        }
    }
}

pub enum ReadResult {
    ReadLength(ssize_t),
    EndOfRead,
}

#[derive(Debug)]
pub(crate) struct OpenSocket {
    fd: c_int,
}

impl OpenSocket {
    pub(crate) fn get_lladdr(&self, ifreq: &mut libc::ifreq) -> Result<()> {
        let fd = self.fd;
        match sys::ioctl(fd, sio::SIOCGIFLLADDR, ifreq.as_mut_ptr()) {
            0 => Ok(()),
            ret => {
                let ifname = ifreq.name();
                let errno = sys::errno();
                Err(Error::GetLinkLevelAddress(fd, ifname, ret, errno).into())
            }
        }
    }

    pub(crate) fn set_lladdr(&self, ifreq: &mut libc::ifreq) -> Result<()> {
        let fd = self.fd;
        match sys::ioctl(fd, sio::SIOCSIFLLADDR, ifreq.as_mut_ptr()) {
            0 => Ok(()),
            ret => {
                let ifname = ifreq.name();
                let lladdr = ifreq.lladdr();
                let errno = sys::errno();
                Err(Error::SetLinkLevelAddress(fd, ifname, lladdr, ret, errno).into())
            }
        }
    }

    pub(crate) fn read(&self, buf: &mut [c_char]) -> Result<ReadResult> {
        let fd = self.fd;
        match sys::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len()) {
            0 => Ok(ReadResult::EndOfRead),
            ret if ret < 0 => {
                let errno = sys::errno();
                Err(Error::Read(fd, ret, errno).into())
            }
            ret => Ok(ReadResult::ReadLength(ret)),
        }
    }
}

impl Drop for OpenSocket {
    fn drop(&mut self) {
        let fd = self.fd;
        match sys::close(fd) {
            0 => (),
            ret => {
                let errno = sys::errno();
                let error = Error::Close(fd, ret, errno);
                eprintln!("Error: {:?}", error);
            }
        };
    }
}

#[cfg(test)]
pub(crate) mod mocks {
    pub(crate) mod sys {
        use libc::{c_int, c_ulong, c_void, size_t, ssize_t};

        use mockdown::{mockdown, Mock};

        use super::super::super::sys;

        pub(crate) use sys::strerror;

        pub(crate) struct Socket(pub fn(domain: c_int, ty: c_int, protocol: c_int) -> c_int);
        pub(crate) struct Ioctl(pub fn(fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int);
        pub(crate) struct Read(pub fn(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t);
        pub(crate) struct Close(pub fn(fd: c_int) -> c_int);
        pub(crate) struct ErrNo(pub fn() -> c_int);

        pub(crate) fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            mockdown()
                .next(|Socket(mock)| mock(domain, ty, protocol))
                .unwrap()
        }

        pub(crate) fn ioctl(fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            mockdown()
                .next(|Ioctl(mock)| mock(fd, request, arg))
                .unwrap()
        }

        pub(crate) fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
            mockdown().next(|Read(mock)| mock(fd, buf, count)).unwrap()
        }

        pub(crate) fn close(fd: c_int) -> c_int {
            mockdown().next(|Close(mock)| mock(fd)).unwrap()
        }

        pub(crate) fn errno() -> c_int {
            mockdown().next(|ErrNo(mock)| mock()).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use libc::c_int;
    use mockdown::{mockdown, Mock};

    use crate::ifname::IfName;
    use crate::lladdr::LinkLevelAddress;
    use crate::Result;

    use super::super::defs::sio;
    use super::super::types::ifreq::tests::PtrAsIfReq;
    use super::super::types::ifreq::{self, IfReq, IfReqMut, IfReqWith};
    use super::{open_local_dgram, OpenSocket};

    use super::mocks::sys;

    const MOCK_FD: c_int = 3;
    const MOCK_SUCCESS: c_int = 0;
    const MOCK_FAILURE: c_int = -1;
    const MOCK_SOCKET: (c_int, c_int, c_int) = (libc::AF_LOCAL, libc::SOCK_DGRAM, 0);

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    #[test]
    fn test_socket_open_local_dgram() -> Result<()> {
        const FD: c_int = 10;

        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                FD
            }))
            .expect(sys::Close(|fd| {
                assert_eq!(FD, fd);
                MOCK_SUCCESS
            }));

        let expected_open_socket = "OpenSocket { fd: 10 }";

        let open_socket = open_local_dgram()?;

        assert_eq!(format!("{:?}", open_socket), expected_open_socket);

        Ok(())
    }

    #[test]
    fn test_socket_open_local_dgram_error() {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FAILURE
            }))
            .expect(sys::ErrNo(|| libc::EPERM));

        let expected_error = "Socket::OpenLocalDgramError { ret: -1, errno: 1, strerror: \"Operation not permitted\" }";

        let error = open_local_dgram().unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_open_socket_debug() {
        mockdown().expect(sys::Close(|fd| {
            assert_eq!(MOCK_FD, fd);
            MOCK_SUCCESS
        }));

        let expected_debug = "OpenSocket { fd: 3 }";

        let box_open_socket = OpenSocket { fd: MOCK_FD };

        assert_eq!(format!("{:?}", box_open_socket), expected_debug);
    }

    #[test]
    fn test_open_socket_get_lladdr() -> Result<()> {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FD
            }))
            .expect(sys::Ioctl(|fd, request, arg| {
                assert_eq!((MOCK_FD, sio::SIOCGIFLLADDR), (fd, request));
                assert_eq!(arg.as_ifreq().name(), *IFNAME);
                arg.as_ifreq().change_lladdr(&LLADDR);
                MOCK_SUCCESS
            }))
            .expect(sys::Close(|fd| {
                assert_eq!(MOCK_FD, fd);
                MOCK_SUCCESS
            }));

        let mut ifreq = ifreq::new().with_name(&IFNAME);

        open_local_dgram()?.get_lladdr(&mut ifreq)?;

        assert_eq!(ifreq.lladdr(), *LLADDR);
        Ok(())
    }

    #[test]
    fn test_open_socket_get_lladdr_error() -> Result<()> {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FD
            }))
            .expect(sys::Ioctl(|fd, request, arg| {
                assert_eq!((MOCK_FD, sio::SIOCGIFLLADDR), (fd, request));
                assert_eq!(arg.as_ifreq().name(), *IFNAME);
                MOCK_FAILURE
            }))
            .expect(sys::ErrNo(|| libc::EBADF))
            .expect(sys::Close(|fd| {
                assert_eq!(MOCK_FD, fd);
                MOCK_SUCCESS
            }));

        let expected_error = "Socket::GetLinkLevelAddressError { fd: 3, ifname: \"enx\", ret: -1, errno: 9, strerror: \"Bad file descriptor\" }";
        let mut ifreq = ifreq::new().with_name(&IFNAME);

        let error = open_local_dgram()?.get_lladdr(&mut ifreq).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr() -> Result<()> {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FD
            }))
            .expect(sys::Ioctl(|fd, request, arg| {
                assert_eq!((MOCK_FD, sio::SIOCSIFLLADDR), (fd, request));
                assert_eq!(arg.as_ifreq().name(), *IFNAME);
                assert_eq!(arg.as_ifreq().lladdr(), *LLADDR);
                MOCK_SUCCESS
            }))
            .expect(sys::Close(|fd| {
                assert_eq!(MOCK_FD, fd);
                MOCK_SUCCESS
            }));

        let mut ifreq = ifreq::new().with_name(&IFNAME).with_lladdr(&LLADDR);

        open_local_dgram()?.set_lladdr(&mut ifreq)?;

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr_error() -> Result<()> {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FD
            }))
            .expect(sys::Ioctl(|fd, request, arg| {
                assert_eq!((MOCK_FD, sio::SIOCSIFLLADDR), (fd, request));
                assert_eq!(arg.as_ifreq().name(), *IFNAME);
                assert_eq!(arg.as_ifreq().lladdr(), *LLADDR);
                MOCK_FAILURE
            }))
            .expect(sys::ErrNo(|| libc::EINVAL))
            .expect(sys::Close(|fd| {
                assert_eq!(MOCK_FD, fd);
                MOCK_SUCCESS
            }));

        let expected_error = "Socket::SetLinkLevelAddressError { fd: 3, ifname: \"enx\", lladdr: \"00:11:22:33:44:55\", ret: -1, errno: 22, strerror: \"Invalid argument\" }";
        let mut ifreq = ifreq::new().with_name(&IFNAME).with_lladdr(&LLADDR);

        let error = open_local_dgram()?.set_lladdr(&mut ifreq).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_close() -> Result<()> {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FD
            }))
            .expect(sys::Close(|fd| {
                assert_eq!(MOCK_FD, fd);
                MOCK_SUCCESS
            }));

        let open_socket = open_local_dgram()?;

        drop(open_socket);

        Ok(())
    }

    #[test]
    fn test_open_socket_close_error() -> crate::Result<()> {
        mockdown()
            .expect(sys::Socket(|domain, ty, protocol| {
                assert_eq!(MOCK_SOCKET, (domain, ty, protocol));
                MOCK_FD
            }))
            .expect(sys::Close(|fd| {
                assert_eq!(MOCK_FD, fd);
                MOCK_FAILURE
            }))
            .expect(sys::ErrNo(|| libc::EINTR));

        let open_socket = open_local_dgram()?;

        drop(open_socket);

        Ok(())
    }
}
