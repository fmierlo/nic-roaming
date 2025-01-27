use std::fmt::{Debug, Display};

use crate::ifname::IfName;
use crate::lladdr::LinkLevelAddress;
use crate::Result;

use super::ifreq::{IfReq, PtrAsIfReq};
#[cfg(not(test))]
use super::sys;

#[cfg(test)]
use mocks::sys;

#[derive(Clone, PartialEq, Eq)]
enum Error {
    OpenLocalDgram(libc::c_int, libc::c_int),
    GetLinkLevelAddress(libc::c_int, IfName, libc::c_int, libc::c_int),
    SetLinkLevelAddress(
        libc::c_int,
        IfName,
        LinkLevelAddress,
        libc::c_int,
        libc::c_int,
    ),
    Close(libc::c_int, libc::c_int, libc::c_int),
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
    match sys::socket(libc::AF_LOCAL, libc::SOCK_DGRAM, 0) {
        fd if fd >= 0 => Ok(OpenSocket { fd }),
        ret => {
            let errno = sys::errno();
            Err(Error::OpenLocalDgram(ret, errno).into())
        }
    }
}

#[derive(Debug)]
pub(crate) struct OpenSocket {
    fd: libc::c_int,
}

impl OpenSocket {
    pub(crate) fn get_lladdr(&self, ifreq_ptr: *mut libc::c_void) -> Result<()> {
        let fd = self.fd;
        match sys::ioctl(fd, sys::SIOCGIFLLADDR, ifreq_ptr) {
            0 => Ok(()),
            ret => {
                let ifname = ifreq_ptr.as_ifreq().name();
                let errno = sys::errno();
                Err(Error::GetLinkLevelAddress(fd, ifname, ret, errno).into())
            }
        }
    }

    pub(crate) fn set_lladdr(&self, ifreq_ptr: *mut libc::c_void) -> Result<()> {
        let fd = self.fd;
        match sys::ioctl(fd, sys::SIOCSIFLLADDR, ifreq_ptr) {
            0 => Ok(()),
            ret => {
                let ifname = ifreq_ptr.as_ifreq().name();
                let lladdr = ifreq_ptr.as_ifreq().lladdr();
                let errno = sys::errno();
                Err(Error::SetLinkLevelAddress(fd, ifname, lladdr, ret, errno).into())
            }
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
        use libc::{c_int, c_ulong, c_void};

        use mockdown::{mockdown, Mock};

        use super::super::super::sys;

        pub(crate) use sys::{strerror, SIOCGIFLLADDR, SIOCSIFLLADDR};

        #[derive(Debug, PartialEq)]
        pub(crate) struct Socket(pub c_int, pub c_int, pub c_int);
        #[derive(Debug, PartialEq)]
        pub(crate) struct IoCtl(pub (c_int, c_ulong), pub *mut c_void);
        #[derive(Debug, PartialEq)]
        pub(crate) struct Close(pub c_int);
        #[derive(Debug)]
        pub(crate) struct ErrNo();

        pub(crate) fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
            let args = Socket(domain, ty, protocol);
            mockdown().mock(args).unwrap()
        }

        pub(crate) fn ioctl(fd: c_int, request: c_ulong, ifreq_ptr: *mut c_void) -> c_int {
            let args = IoCtl((fd, request), ifreq_ptr);
            mockdown().mock(args).unwrap()
        }

        pub(crate) fn close(fd: c_int) -> c_int {
            let args = Close(fd);
            mockdown().mock(args).unwrap()
        }

        pub(crate) fn errno() -> c_int {
            let args = ErrNo();
            mockdown().mock(args).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use mockdown::{mockdown, Mock};

    use crate::ifname::IfName;
    use crate::ifreq::{self, IfReq, IfReqAsPtr, IfReqMut, IfReqWith, PtrAsIfReq};
    use crate::lladdr::LinkLevelAddress;
    use crate::Result;

    use super::{open_local_dgram, OpenSocket};

    use super::mocks::sys;

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    const MOCK_FD: libc::c_int = 3;

    const RETURN_FD: libc::c_int = MOCK_FD;
    const RETURN_SUCCESS: libc::c_int = 0;
    const RETURN_FAILURE: libc::c_int = -1;

    const MOCK_SOCKET: sys::Socket = sys::Socket(libc::AF_LOCAL, libc::SOCK_DGRAM, 0);
    const MOCK_CLOSE: sys::Close = sys::Close(MOCK_FD);

    #[test]
    fn test_socket_open_local_dgram() {
        const FD: libc::c_int = 10;
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                FD
            })
            .expect(|args| {
                assert_eq!(sys::Close(FD), args);
                RETURN_SUCCESS
            });

        let expected_open_socket = "OpenSocket { fd: 10 }";

        let open_socket = open_local_dgram().unwrap();

        assert_eq!(format!("{:?}", open_socket), expected_open_socket);
    }

    #[test]
    fn test_socket_open_local_dgram_error() {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FAILURE
            })
            .expect(|_: sys::ErrNo| {
                assert!(true);
                libc::EPERM
            });

        let expected_error = "Socket::OpenLocalDgramError { ret: -1, errno: 1, strerror: \"Operation not permitted\" }";

        let error = open_local_dgram().unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_open_socket_debug() {
        mockdown().expect(|args| {
            assert_eq!(MOCK_CLOSE, args);
            RETURN_SUCCESS
        });

        let expected_debug = "OpenSocket { fd: 3 }";

        let box_open_socket = OpenSocket { fd: MOCK_FD };

        assert_eq!(format!("{:?}", box_open_socket), expected_debug);
    }

    #[test]
    fn test_open_socket_get_lladdr() -> Result<()> {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq_ptr)| {
                assert_eq!((MOCK_FD, sys::SIOCGIFLLADDR), args);
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                ifreq_ptr.as_ifreq().change_lladdr(&LLADDR);
                RETURN_SUCCESS
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let mut ifreq = ifreq::new().with_name(&IFNAME);

        open_local_dgram()?.get_lladdr(ifreq.as_mut_ptr()).unwrap();

        assert_eq!(ifreq.lladdr(), *LLADDR);
        Ok(())
    }

    #[test]
    fn test_open_socket_get_lladdr_error() -> Result<()> {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq_ptr)| {
                assert_eq!((MOCK_FD, sys::SIOCGIFLLADDR), args);
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                RETURN_FAILURE
            })
            .expect(|_: sys::ErrNo| {
                assert!(true);
                libc::EBADF
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let expected_error = "Socket::GetLinkLevelAddressError { fd: 3, ifname: \"enx\", ret: -1, errno: 9, strerror: \"Bad file descriptor\" }";
        let mut ifreq = ifreq::new().with_name(&IFNAME);

        let error = open_local_dgram()?
            .get_lladdr(ifreq.as_mut_ptr())
            .unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr() -> Result<()> {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq_ptr)| {
                assert_eq!((MOCK_FD, sys::SIOCSIFLLADDR), args);
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                assert_eq!(ifreq_ptr.as_ifreq().lladdr(), *LLADDR);
                RETURN_SUCCESS
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let mut ifreq = ifreq::new().with_name(&IFNAME).with_lladdr(&LLADDR);

        open_local_dgram()?.set_lladdr(ifreq.as_mut_ptr())?;

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr_error() -> Result<()> {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq_ptr)| {
                assert_eq!((MOCK_FD, sys::SIOCSIFLLADDR), args);
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                assert_eq!(ifreq_ptr.as_ifreq().lladdr(), *LLADDR);
                RETURN_FAILURE
            })
            .expect(|_: sys::ErrNo| {
                assert!(true);
                libc::EINVAL
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let expected_error = "Socket::SetLinkLevelAddressError { fd: 3, ifname: \"enx\", lladdr: \"00:11:22:33:44:55\", ret: -1, errno: 22, strerror: \"Invalid argument\" }";
        let mut ifreq = ifreq::new().with_name(&IFNAME).with_lladdr(&LLADDR);

        let error = open_local_dgram()?
            .set_lladdr(ifreq.as_mut_ptr())
            .unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_close() {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let open_socket = open_local_dgram().unwrap();

        drop(open_socket);
    }

    #[test]
    fn test_open_socket_close_error() {
        mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_FAILURE
            })
            .expect(|_: sys::ErrNo| {
                assert!(true);
                libc::EINTR
            });

        let open_socket = open_local_dgram().unwrap();

        drop(open_socket);
    }
}
