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

pub(super) trait OpenSocket {
    fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()>;
    fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()>;
}

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
        let result = match self.close(fd) {
            0 => Ok(()),
            ret => {
                let errno = self.errno();
                Err(Error::Close(fd, ret, errno))
            }
        };

        if let Err(error) = result {
            eprintln!("Error: {:?}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::sys::mock::MockSys;
    use super::{ifreq, BoxSys, IfName, LibcSocket, LinkLevelAddress, Result, Socket};

    impl<'a> LibcSocket {
        fn new(sys: &MockSys) -> LibcSocket {
            LibcSocket(BoxSys(Box::new(sys.clone())))
        }
    }

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

    // #[test]
    // fn test_socket_open_local_dgram() -> Result<()> {
    //     let sys = MockSys::default();
    //     let socket = Socket::new(sys.as_sys());
    //     let fd = socket.open_local_dgram()?;
    //     assert!(!fd.is_null());
    //     Ok(())
    // }

    // #[test]
    // fn test_socket_open_local_dgram_err() -> Result<()> {
    //     let mut sys = MockSys::default();
    //     // Set the error code to -1
    //     sys.set_last_os_error(ErrorCode::last_os_error());
    //     let socket = Socket::new(sys.as_sys());
    //     let fd = socket.open_local_dgram()?;
    //     assert_eq!(fd, -1);
    //     Ok(())
    // }

    #[test]
    fn test_local_dgram_socket_get_lladdr() -> Result<()> {
        let ifname: IfName = "enx".try_into()?;
        let expected_lladdr: LinkLevelAddress = "00:11:22:33:44:55".parse()?;
        let sys = MockSys::default().with_nic(ifname, expected_lladdr);
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &ifname);

        LibcSocket::new(&sys)
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        assert_eq!(ifreq::get_lladdr(&ifreq), expected_lladdr);
        Ok(())
    }

    // #[test]
    // fn test_local_dgram_socket_get_lladdr_err() -> Result<()> {
    //     let mut sys = MockSys::default();
    //     // Set the error code to -1
    //     sys.set_last_os_error(ErrorCode::last_os_error());
    //     let socket = Socket::new(sys.as_sys());
    //     let fd = socket.open_local_dgram()?;
    //     assert_eq!(
    //         socket.get_lladdr(&mut [0; 16])?,
    //         Err(Error::last_os_error())
    //     );
    //     Ok(())
    // }

    #[test]
    fn test_local_dgram_socket_set_lladdr() -> Result<()> {
        let ifname: IfName = "enx".try_into()?;
        let lladdr: LinkLevelAddress = "00:11:22:33:44:55".parse()?;
        let sys = MockSys::default();
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &ifname);
        ifreq::set_lladdr(&mut ifreq, &lladdr);

        LibcSocket::new(&sys)
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        assert!(sys.has_nic(&ifname, &lladdr));
        Ok(())
    }

    // #[test]
    // fn test_local_dgram_socket_set_lladdr_err() -> Result<()> {
    //     let mut sys = MockSys::default();
    //     // Set the error code to -1
    //     sys.set_last_os_error(ErrorCode::last_os_error());
    //     let socket = Socket::new(sys.as_sys());
    //     let fd = socket.open_local_dgram()?;
    //     assert_eq!(
    //         socket.set_lladdr(&mut [0; 16])?,
    //         Err(Error::last_os_error())
    //     );
    //     Ok(())
    // }

    // #[test]
    // fn test_socket_close() {
    //     let sys = MockSys::default();
    //     // Create a dummy local dgram socket
    //     let socket = Socket::new(sys.as_sys());
    //     let fd = socket.open_local_dgram()?;
    //     assert!(!fd.is_null());
    //     drop(socket); // Close the socket
    // }
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
