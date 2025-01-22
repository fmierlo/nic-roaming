use crate::{IfName, LinkLevelAddress, Result};

use super::ifreq;

#[cfg(not(test))]
use super::socket;

#[cfg(test)]
use mocks::socket;

#[derive(Debug, Default)]
pub struct Nic {
    socket: BoxSocket,
}

impl Nic {
    pub fn get_lladd(&self, ifname: &IfName) -> Result<LinkLevelAddress> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, ifname);

        self.socket
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        Ok(ifreq::get_lladdr(&ifreq))
    }

    pub fn set_lladd(&self, ifname: &IfName, lladdr: &LinkLevelAddress) -> Result<()> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, ifname);
        ifreq::set_lladdr(&mut ifreq, lladdr);

        self.socket
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))
    }
}

#[cfg(test)]
pub(crate) mod mocks {
    pub(crate) mod socket {
        use std::fmt::Debug;

        use mockdown::{mockdown, Mock};

        use crate::Result;

        thread_local! {
            static MOCKDOWN: RefCell<Mockdown> = Mockdown::thread_local();
        }

        pub(crate) fn mockdown() -> &'static LocalKey<RefCell<Mockdown>> {
            &MOCKDOWN
        }

        #[derive(Debug, PartialEq)]
        pub(crate) struct OpenLocalDgram();
        #[derive(Debug, PartialEq)]
        pub(crate) struct GetLLAddr(pub *mut libc::c_void);
        #[derive(Debug, PartialEq)]
        pub(crate) struct SetLLAddr(pub *mut libc::c_void);

        pub(crate) struct MockResult {}

        impl MockResult {
            pub(crate) fn ok() -> Result<()> {
                Result::<()>::Ok(())
            }

            pub(crate) fn err(error: &str) -> Result<()> {
                Result::<()>::Err(error.into())
            }
        }

        #[derive(Clone, Debug, Default)]
        pub(crate) struct MockSocket();

        impl Socket for MockSocket {
            fn open_local_dgram(&self) -> SocketResult {
                let args = OpenLocalDgram();
                MOCKDOWN.mock(args)?
            }
        }

        #[derive(Debug)]
        pub(crate) struct MockOpenSocket();

        impl MockOpenSocket {
            pub(crate) fn ok() -> SocketResult {
                SocketResult::Ok(Box::new(MockOpenSocket()))
            }

            pub(crate) fn err(error: &str) -> SocketResult {
                SocketResult::Err(error.into())
            }
        }

        impl OpenSocket for MockOpenSocket {
            fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
                let args = GetLLAddr(arg);
                MOCKDOWN.mock(args)?
            }

            fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
                let args = SetLLAddr(arg);
                MOCKDOWN.mock(args)?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use mockdown::{mockdown, Mock};

    use crate::{IfName, LinkLevelAddress, Nic};

    use super::ifreq::mock::{ifreq_get_lladdr, ifreq_get_name, ifreq_set_lladdr};

    use super::mocks::socket::{self, MockResult, OpenSocket};

    impl Nic {
        fn new(socket: &MockSocket) -> Nic {
            Nic {
                socket: BoxSocket(Box::new(socket.clone())),
            }
        }
    }

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    #[test]
    fn test_nic_default() {
        let expected_default = "Nic { socket: BoxSocket(LibcSocket) }";

        let nic = super::Nic::default();

        assert_eq!(format!("{:?}", nic), expected_default);
    }

    #[test]
    fn test_get_lladdr() {
        socket::mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                MockOpenSocket::ok()
            })
            .expect(|socket::GetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                MockResult::ok()
            });

        let socket = MockSocket::default();
        let lladdr = Nic::new(&socket).get_lladd(&IFNAME).unwrap();

        assert_eq!(lladdr, *LLADDR);
    }

    #[test]
    fn test_get_lladdr_open_error() {
        socket::mockdown().expect(|socket::OpenLocalDgram()| {
            assert!(true);
            MockOpenSocket::err("GetLinkLevelAddressOpenError")
        });

        let expected_error = "GetLinkLevelAddressOpenError";

        let socket = MockSocket::default();
        let error = Nic::new(&socket).get_lladd(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_get_lladdr_error() {
        socket::mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                MockOpenSocket::ok()
            })
            .expect(|socket::GetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                MockResult::err("GetLinkLevelAddressError")
            });

        let expected_error = "GetLinkLevelAddressError";

        let socket = MockSocket::default();
        let error = Nic::new(&socket).get_lladd(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr() {
        socket::mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                MockOpenSocket::ok()
            })
            .expect(|socket::SetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                MockResult::ok()
            });

        let socket = MockSocket::default();
        Nic::new(&socket).set_lladd(&IFNAME, &LLADDR).unwrap();
    }

    #[test]
    fn test_set_lladdr_open_error() {
        socket::mockdown().expect(|socket::OpenLocalDgram()| {
            assert!(true);
            MockOpenSocket::err("SetLinkLevelAddressOpenError")
        });

        let expected_error = "SetLinkLevelAddressOpenError";

        let socket = MockSocket::default();
        let error = Nic::new(&socket).set_lladd(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr_error() {
        socket::mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                MockOpenSocket::ok()
            })
            .expect(|socket::SetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                MockResult::err("SetLinkLevelAddressError")
            });

        let expected_error = "SetLinkLevelAddressError";

        let socket = MockSocket::default();
        let error = Nic::new(&socket).set_lladd(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }
}
