use super::ifname::IfName;
use super::ifreq::{self};
use super::sys::{self, BoxSys};
use crate::{LinkLevelAddress, Result};
use std::fmt::{Debug, Display};
use std::ops::Deref;

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

pub(super) trait Socket: Debug {
    fn open_local_dgram(&self) -> Result<Box<dyn OpenSocket + '_>>;
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
struct LibcSocket(BoxSys);

impl Deref for LibcSocket {
    type Target = BoxSys;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Socket for LibcSocket {
    fn open_local_dgram(&self) -> Result<Box<dyn OpenSocket + '_>> {
        match self.socket(libc::AF_LOCAL, libc::SOCK_DGRAM, 0) {
            fd if fd >= 0 => Ok(Box::new(LibcOpenSocket { fd, sys: self })),
            ret => {
                let errno = self.errno();
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
struct LibcOpenSocket<'a> {
    fd: libc::c_int,
    sys: &'a BoxSys,
}

impl<'a> Deref for LibcOpenSocket<'a> {
    type Target = &'a BoxSys;

    fn deref(&self) -> &Self::Target {
        &self.sys
    }
}

impl<'a> OpenSocket for LibcOpenSocket<'a> {
    fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
        let fd = self.fd;
        match self.ioctl(fd, sys::SIOCGIFLLADDR, arg) {
            0 => Ok(()),
            ret => {
                let ifreq = ifreq::from_mut_ptr(arg);
                let ifname = ifreq::get_name(ifreq);
                let errno = self.errno();
                Err(Error::GetLinkLevelAddress(fd, ifname, ret, errno).into())
            }
        }
    }

    fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
        let fd = self.fd;
        match self.ioctl(fd, sys::SIOCSIFLLADDR, arg) {
            0 => Ok(()),
            ret => {
                let ifreq = ifreq::from_mut_ptr(arg);
                let ifname = ifreq::get_name(ifreq);
                let lladdr = ifreq::get_lladdr(ifreq);
                let errno = self.errno();
                Err(Error::SetLinkLevelAddress(fd, ifname, lladdr, ret, errno).into())
            }
        }
    }
}

impl<'a> Drop for LibcOpenSocket<'a> {
    fn drop(&mut self) {
        let fd = self.fd;
        match self.close(fd) {
            0 => (),
            ret => {
                let errno = self.errno();
                let error = Error::Close(fd, ret, errno);
                eprintln!("Error: {:?}", error);
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::super::sys::mock::{self, MockSys};
    use super::{ifreq, BoxSys, IfName, LibcSocket, LinkLevelAddress, Result, Socket};

    impl<'a> LibcSocket {
        fn new(sys: &MockSys) -> LibcSocket {
            LibcSocket(BoxSys(Box::new(sys.clone())))
        }
    }

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    const MOCK_SUCCESS: libc::c_int = 0;
    const MOCK_FAILURE: libc::c_int = -1;
    const MOCK_FD: libc::c_int = 3;
    const MOCK_SOCKET: mock::Socket = mock::Socket((libc::AF_LOCAL, libc::SOCK_DGRAM, 0), MOCK_FD);
    const MOCK_CLOSE: mock::Close = mock::Close((MOCK_FD,), MOCK_SUCCESS);

    #[test]
    fn test_socket_box_default() {
        let expected_default = "BoxSocket(LibcSocket(BoxSys(LibcSys)))";

        let box_socket = super::BoxSocket::default();

        assert_eq!(format!("{:?}", box_socket), expected_default);
    }

    #[test]
    fn test_socket_box_debug() {
        let socket = super::mock::MockSocket::default();
        let expected_debug = "BoxSocket(MockSocket { kv: RefCell { value: {} } })";

        let box_socket = super::BoxSocket(Box::new(socket));

        assert_eq!(format!("{:?}", box_socket), expected_debug);
    }

    #[test]
    fn test_socket_box_deref() {
        let socket = super::mock::MockSocket::default();
        let expected_deref = "MockSocket { kv: RefCell { value: {} } }";

        let deref_box_socket = &*super::BoxSocket(Box::new(socket));

        assert_eq!(format!("{:?}", deref_box_socket), expected_deref);
    }

    #[test]
    fn test_open_socket_box_debug() {
        let sys = &BoxSys(Box::new(MockSys::default().on(MOCK_CLOSE)));

        let expected_debug = "LibcOpenSocket { fd: 3, sys: BoxSys(MockSys { mock: Mock }) }";

        let box_open_socket: Box<dyn super::OpenSocket> =
            Box::new(super::LibcOpenSocket { fd: MOCK_FD, sys });

        assert_eq!(format!("{:?}", box_open_socket), expected_debug);
    }

    #[test]
    fn test_socket_open_local_dgram() {
        let sys = MockSys::default()
            .on(mock::Socket(MOCK_SOCKET.0, 10))
            .on(mock::Close((10,), MOCK_SUCCESS));

        let expected_open_socket = "LibcOpenSocket { fd: 10, sys: BoxSys(MockSys { mock: Mock }) }";
        let socket = LibcSocket::new(&sys);

        let open_socket = socket.open_local_dgram().unwrap();

        assert_eq!(format!("{:?}", open_socket), expected_open_socket);
    }

    #[test]
    fn test_socket_open_local_dgram_error() {
        let sys = MockSys::default()
            .on(mock::Socket(MOCK_SOCKET.0, MOCK_FAILURE))
            .on(mock::ErrNo((), libc::EPERM));

        let expected_error = "Socket::OpenLocalDgramError { ret: -1, errno: 1, strerror: \"Operation not permitted\" }";
        let socket = LibcSocket::new(&sys);

        let error = socket.open_local_dgram().unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_open_socket_get_lladdr() -> Result<()> {
        let sys = MockSys::default()
            .on(MOCK_SOCKET)
            .on(mock::IoCtl(
                (MOCK_FD, super::sys::SIOCGIFLLADDR, *IFNAME, None),
                (MOCK_SUCCESS, Some(*LLADDR)),
            ))
            .on(MOCK_CLOSE);

        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);

        LibcSocket::new(&sys)
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))
            .unwrap();

        assert_eq!(ifreq::get_lladdr(&ifreq), *LLADDR);
        Ok(())
    }

    #[test]
    fn test_open_socket_get_lladdr_error() -> Result<()> {
        let sys = MockSys::default()
            .on(MOCK_SOCKET)
            .on(mock::IoCtl(
                (MOCK_FD, super::sys::SIOCGIFLLADDR, *IFNAME, None),
                (MOCK_FAILURE, None),
            ))
            .on(mock::ErrNo((), libc::EBADF))
            .on(MOCK_CLOSE);

        let expected_error = "Socket::GetLinkLevelAddressError { fd: 3, ifname: \"enx\", ret: -1, errno: 9, strerror: \"Bad file descriptor\" }";
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);

        let error = LibcSocket::new(&sys)
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))
            .unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr() -> Result<()> {
        let sys = MockSys::default()
            .on(MOCK_SOCKET)
            .on(mock::IoCtl(
                (MOCK_FD, super::sys::SIOCSIFLLADDR, *IFNAME, Some(*LLADDR)),
                (MOCK_SUCCESS, None),
            ))
            .on(MOCK_CLOSE);

        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);
        ifreq::set_lladdr(&mut ifreq, &LLADDR);

        LibcSocket::new(&sys)
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        Ok(())
    }

    #[test]
    fn test_open_socket_set_lladdr_error() -> Result<()> {
        let sys = MockSys::default()
            .on(MOCK_SOCKET)
            .on(mock::IoCtl(
                (MOCK_FD, super::sys::SIOCSIFLLADDR, *IFNAME, Some(*LLADDR)),
                (MOCK_FAILURE, None),
            ))
            .on(mock::ErrNo((), libc::EINVAL))
            .on(MOCK_CLOSE);

        let expected_error = "Socket::SetLinkLevelAddressError { fd: 3, ifname: \"enx\", lladdr: \"00:11:22:33:44:55\", ret: -1, errno: 22, strerror: \"Invalid argument\" }";
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &IFNAME);
        ifreq::set_lladdr(&mut ifreq, &LLADDR);

        let error = LibcSocket::new(&sys)
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))
            .unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);

        Ok(())
    }

    #[test]
    fn test_open_socket_close() {
        let sys = MockSys::default().on(MOCK_SOCKET).on(MOCK_CLOSE);

        let socket = LibcSocket::new(&sys);

        let open_socket = socket.open_local_dgram().unwrap();

        drop(open_socket);
    }

    #[test]
    fn test_open_socket_close_error() {
        let sys = MockSys::default()
            .on(MOCK_SOCKET)
            .on(mock::Close(MOCK_CLOSE.0, MOCK_FAILURE))
            .on(mock::ErrNo((), libc::EINTR));

        let socket = LibcSocket::new(&sys);

        let open_socket = socket.open_local_dgram().unwrap();

        drop(open_socket);
    }
}

#[cfg(test)]
pub(super) mod mock {
    use super::super::ifname::IfName;
    use super::ifreq::{self};
    use super::{OpenSocket, Socket};
    use crate::{LinkLevelAddress, Result};
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    type KeyValue = RefCell<HashMap<IfName, LinkLevelAddress>>;

    #[derive(Clone, Debug, Default)]
    pub(crate) struct MockSocket {
        kv: Rc<KeyValue>,
    }

    impl MockSocket {
        pub(crate) fn with_nic(self, ifname: IfName, lladdr: LinkLevelAddress) -> Self {
            self.set_nic(ifname, lladdr);
            self
        }

        pub(crate) fn set_nic(&self, ifname: IfName, lladdr: LinkLevelAddress) {
            self.kv.borrow_mut().insert(ifname, lladdr);
        }

        pub(crate) fn has_nic(&self, ifname: &IfName, expected_lladdr: &LinkLevelAddress) -> bool {
            match self.kv.borrow().get(ifname) {
                Some(lladdr) => lladdr == expected_lladdr,
                None => false,
            }
        }
    }

    impl Socket for MockSocket {
        fn open_local_dgram(&self) -> Result<Box<dyn OpenSocket + '_>> {
            eprintln!("MockSocket.open_local_dgram()");
            Ok(Box::new(MockOpenSocket { kv: &self.kv }))
        }
    }

    #[derive(Debug)]
    pub(super) struct MockOpenSocket<'a> {
        kv: &'a Rc<KeyValue>,
    }

    impl<'a> OpenSocket for MockOpenSocket<'a> {
        fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
            let ifreq = ifreq::from_mut_ptr(arg);
            let ifname = ifreq::get_name(ifreq);

            if let Some(lladdr) = self.kv.borrow().get(&ifname) {
                eprintln!("MockOpenSocket.get_lladdr({ifname}) -> {lladdr})");
                ifreq::set_lladdr(ifreq, lladdr)
            };
            Ok(())
        }

        fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
            let ifreq = ifreq::from_mut_ptr(arg);
            let ifname = ifreq::get_name(ifreq);
            let lladdr = ifreq::get_lladdr(ifreq);

            eprintln!("MockOpenSocket.set_lladdr({ifname}, {lladdr})");
            self.kv.borrow_mut().insert(ifname, lladdr);

            Ok(())
        }
    }
}
