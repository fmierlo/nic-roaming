use super::ifname::IfName;
use super::ifreq::{self};
use crate::{LinkLevelAddress, Result};
use std::fmt::{Debug, Display};
use std::ops::Deref;

#[cfg(not(test))]
use super::sys;

#[cfg(test)]
use mocks::sys;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Error {
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

pub(crate) type SocketResult = Result<Box<dyn OpenSocket>>;

pub(super) trait Socket: Debug {
    fn open_local_dgram(&self) -> SocketResult;
}

#[derive(Debug, Default)]
pub(super) struct BoxSocket(pub(super) Box<dyn Socket>);

impl Default for Box<dyn Socket> {
    fn default() -> Self {
        Box::new(LibcSocket::default())
    }
}

impl Deref for BoxSocket {
    type Target = Box<dyn Socket>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
struct LibcSocket();

impl Socket for LibcSocket {
    fn open_local_dgram(&self) -> SocketResult {
        match sys::socket(libc::AF_LOCAL, libc::SOCK_DGRAM, 0) {
            fd if fd >= 0 => Ok(Box::new(LibcOpenSocket { fd })),
            ret => {
                let errno = sys::errno();
                Err(Error::OpenLocalDgram(ret, errno).into())
            }
        }
    }
}

pub(super) trait OpenSocket: Debug {
    fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()>;
    fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()>;
}

#[derive(Debug)]
struct LibcOpenSocket {
    fd: libc::c_int,
}

impl OpenSocket for LibcOpenSocket {
    fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
        let fd = self.fd;
        match sys::ioctl(fd, sys::SIOCGIFLLADDR, arg) {
            0 => Ok(()),
            ret => {
                let ifreq = ifreq::from_mut_ptr(arg);
                let ifname = ifreq::get_name(ifreq);
                let errno = sys::errno();
                Err(Error::GetLinkLevelAddress(fd, ifname, ret, errno).into())
            }
        }
    }

    fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
        let fd = self.fd;
        match sys::ioctl(fd, sys::SIOCSIFLLADDR, arg) {
            0 => Ok(()),
            ret => {
                let ifreq = ifreq::from_mut_ptr(arg);
                let ifname = ifreq::get_name(ifreq);
                let lladdr = ifreq::get_lladdr(ifreq);
                let errno = sys::errno();
                Err(Error::SetLinkLevelAddress(fd, ifname, lladdr, ret, errno).into())
            }
        }
    }
}

impl Drop for LibcOpenSocket {
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
        use crate::sys::os::sys;
        use libc::{c_int, c_ulong, c_void};
        use mockdown::Mockdown;
        use mockdown::Static;
        use std::{cell::RefCell, thread::LocalKey};

        pub(crate) use sys::{strerror, SIOCGIFLLADDR, SIOCSIFLLADDR};

        thread_local! {
            static MOCKDOWN: RefCell<Mockdown> = Mockdown::thread_local();
        }

        pub(crate) fn mockdown() -> &'static LocalKey<RefCell<Mockdown>> {
            &MOCKDOWN
        }

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
            MOCKDOWN.mock(args).unwrap()
        }

        pub(crate) fn ioctl(fd: c_int, request: c_ulong, arg: *mut c_void) -> c_int {
            let args = IoCtl((fd, request), arg);
            MOCKDOWN.mock(args).unwrap()
        }

        pub(crate) fn close(fd: c_int) -> c_int {
            let args = Close(fd);
            MOCKDOWN.mock(args).unwrap()
        }

        pub(crate) fn errno() -> c_int {
            let args = ErrNo();
            MOCKDOWN.mock(args).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mocks::sys;
    use super::{ifreq, IfName, LibcSocket, LinkLevelAddress, Result, Socket};
    use crate::sys::os::ifreq::mock::{ifreq_get_lladdr, ifreq_get_name, ifreq_set_lladdr};
    use mockdown::Static;
    use std::sync::LazyLock;

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
    fn test_socket_box_default() {
        let expected_default = "BoxSocket(LibcSocket)";

        let box_socket = super::BoxSocket::default();

        assert_eq!(format!("{:?}", box_socket), expected_default);
    }

    #[test]
    fn test_socket_box_debug() {
        let socket = super::LibcSocket::default();
        let expected_debug = "BoxSocket(LibcSocket)";

        let box_socket = super::BoxSocket(Box::new(socket));

        assert_eq!(format!("{:?}", box_socket), expected_debug);
    }

    #[test]
    fn test_socket_box_deref() {
        let socket = super::LibcSocket::default();
        let expected_deref = "LibcSocket";

        let deref_box_socket = &*super::BoxSocket(Box::new(socket));

        assert_eq!(format!("{:?}", deref_box_socket), expected_deref);
    }

    #[test]
    fn test_open_socket_box_debug() {
        sys::mockdown().expect(|args| {
            assert_eq!(MOCK_CLOSE, args);
            RETURN_SUCCESS
        });

        let expected_debug = "LibcOpenSocket { fd: 3 }";

        let box_open_socket: Box<dyn super::OpenSocket> =
            Box::new(super::LibcOpenSocket { fd: MOCK_FD });

        assert_eq!(format!("{:?}", box_open_socket), expected_debug);
    }

    #[test]
    fn test_socket_open_local_dgram() {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                10
            })
            .expect(|args| {
                assert_eq!(sys::Close(10), args);
                RETURN_SUCCESS
            });

        let expected_open_socket = "LibcOpenSocket { fd: 10 }";

        let open_socket = LibcSocket().open_local_dgram().unwrap();

        assert_eq!(format!("{:?}", open_socket), expected_open_socket);
    }

    #[test]
    fn test_socket_open_local_dgram_error() {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FAILURE
            })
            .expect(|_: sys::ErrNo| {
                assert!(true);
                libc::EPERM
            });

        let expected_error = "Socket::OpenLocalDgramError { ret: -1, errno: 1, strerror: \"Operation not permitted\" }";

        let error = LibcSocket().open_local_dgram().unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_open_socket_get_lladdr() -> Result<()> {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq)| {
                assert_eq!((MOCK_FD, super::sys::SIOCGIFLLADDR), args);
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                RETURN_SUCCESS
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);

        LibcSocket()
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))
            .unwrap();

        assert_eq!(ifreq::get_lladdr(&ifreq), *LLADDR);
        Ok(())
    }

    #[test]
    fn test_open_socket_get_lladdr_error() -> Result<()> {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq)| {
                assert_eq!((MOCK_FD, super::sys::SIOCGIFLLADDR), args);
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
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
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);

        let error = LibcSocket()
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))
            .unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr() -> Result<()> {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq)| {
                assert_eq!((MOCK_FD, super::sys::SIOCSIFLLADDR), args);
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                RETURN_SUCCESS
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);
        ifreq::set_lladdr(&mut ifreq, &LLADDR);

        LibcSocket()
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr_error() -> Result<()> {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|sys::IoCtl(args, ifreq)| {
                assert_eq!((MOCK_FD, super::sys::SIOCSIFLLADDR), args);
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
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
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);
        ifreq::set_lladdr(&mut ifreq, &LLADDR);

        let error = LibcSocket()
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))
            .unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_close() {
        sys::mockdown()
            .expect(|args| {
                assert_eq!(MOCK_SOCKET, args);
                RETURN_FD
            })
            .expect(|args| {
                assert_eq!(MOCK_CLOSE, args);
                RETURN_SUCCESS
            });

        let socket = LibcSocket();

        let open_socket = socket.open_local_dgram().unwrap();

        drop(open_socket);
    }

    #[test]
    fn test_open_socket_close_error() {
        sys::mockdown()
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

        let socket = LibcSocket();

        let open_socket = socket.open_local_dgram().unwrap();

        drop(open_socket);
    }
}
