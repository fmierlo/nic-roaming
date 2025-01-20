use super::ifname::IfName;
use super::ifreq::{self};
use super::socket::BoxSocket;
use crate::{LinkLevelAddress, Result};

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
        use crate::sys::os::socket::{Error, OpenSocket, Socket, SocketResult};
        use crate::Result;
        use mockdown::{Mockdown, Static};
        use std::{cell::RefCell, thread::LocalKey};

        thread_local! {
            static MOCKDOWN: RefCell<Mockdown> = Mockdown::thread_local();
        }

        pub(crate) fn mockdown() -> &'static LocalKey<RefCell<Mockdown>> {
            &MOCKDOWN
        }

        #[derive(Debug, PartialEq)]
        pub(crate) struct OpenLocalDgram();
        pub(crate) type ErrNo = Option<i32>;

        #[derive(Debug, PartialEq)]
        pub(crate) struct GetLLAddr(pub *mut libc::c_void);
        #[derive(Debug, PartialEq)]
        pub(crate) struct SetLLAddr(pub *mut libc::c_void);

        #[derive(Clone, Debug, Default)]
        pub(crate) struct MockSocket();

        impl Socket for MockSocket {
            fn open_local_dgram(&self) -> SocketResult {
                let args = OpenLocalDgram();
                let on_mock: ErrNo = MOCKDOWN.mock(args).unwrap();
                match on_mock {
                    None => Ok(Box::new(MockOpenSocket())),
                    Some(errno) => Err(Error::OpenLocalDgram(-1, errno).into()),
                }
            }
        }

        #[derive(Debug)]
        pub(crate) struct MockOpenSocket();

        impl OpenSocket for MockOpenSocket {
            fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
                let args = GetLLAddr(arg);
                MOCKDOWN.mock(args).unwrap()
            }

            fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
                let args = SetLLAddr(arg);
                MOCKDOWN.mock(args).unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mocks::socket::{self, ErrNo, MockSocket};
    use super::{BoxSocket, IfName, Nic};
    use crate::sys::os::ifreq::mock::{ifreq_get_lladdr, ifreq_get_name, ifreq_set_lladdr};
    use crate::{LinkLevelAddress, Result};
    use mockdown::Static;
    use std::sync::LazyLock;

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
    fn test_get_lladd() {
        socket::mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                ErrNo::None
            })
            .expect(|socket::GetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                Result::Ok(())
            });

        let socket = MockSocket::default();
        let lladdr = Nic::new(&socket).get_lladd(&IFNAME).unwrap();

        assert_eq!(lladdr, *LLADDR);
    }

    #[test]
    fn test_set_lladd() {
        socket::mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                ErrNo::None
            })
            .expect(|socket::SetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                Result::Ok(())
            });

        let socket = MockSocket::default();
        Nic::new(&socket).set_lladd(&IFNAME, &LLADDR).unwrap();
    }
}
