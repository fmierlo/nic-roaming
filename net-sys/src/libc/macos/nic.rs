use crate::{IfName, LinkLevelAddress, Result};

use super::ifreq;

#[cfg(not(test))]
use super::socket;

#[cfg(test)]
use mocks::socket;

pub struct Nic();

impl Nic {
    pub fn new() -> Self {
        Self()
    }

    pub fn get_lladd(&self, ifname: &IfName) -> Result<LinkLevelAddress> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, ifname);

        socket::open_local_dgram()?.get_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        Ok(ifreq::get_lladdr(&ifreq))
    }

    pub fn set_lladd(&self, ifname: &IfName, lladdr: &LinkLevelAddress) -> Result<()> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, ifname);
        ifreq::set_lladdr(&mut ifreq, lladdr);

        socket::open_local_dgram()?.set_lladdr(ifreq::as_mut_ptr(&mut ifreq))
    }
}

#[cfg(test)]
pub(crate) mod mocks {
    pub(crate) mod socket {
        use std::fmt::Debug;

        use mockdown::{mockdown, Mock};

        use crate::Result;

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

        pub(crate) fn open_local_dgram() -> Result<OpenSocket> {
            let args = OpenLocalDgram();
            mockdown().mock(args)?
        }

        #[derive(Debug)]
        pub(crate) struct OpenSocket();

        impl OpenSocket {
            pub(crate) fn ok() -> Result<OpenSocket> {
                Result::<OpenSocket>::Ok(OpenSocket())
            }

            pub(crate) fn err(error: &str) -> Result<OpenSocket> {
                Result::<OpenSocket>::Err(error.into())
            }

            pub(crate) fn get_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
                let args = GetLLAddr(arg);
                mockdown().mock(args)?
            }

            pub(crate) fn set_lladdr(&self, arg: *mut libc::c_void) -> Result<()> {
                let args = SetLLAddr(arg);
                mockdown().mock(args)?
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

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    #[test]
    fn test_get_lladdr() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::GetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                MockResult::ok()
            });

        let lladdr = Nic::new().get_lladd(&IFNAME).unwrap();

        assert_eq!(lladdr, *LLADDR);
    }

    #[test]
    fn test_get_lladdr_open_error() {
        mockdown().expect(|socket::OpenLocalDgram()| {
            assert!(true);
            OpenSocket::err("GetLinkLevelAddressOpenError")
        });

        let expected_error = "GetLinkLevelAddressOpenError";

        let error = Nic::new().get_lladd(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_get_lladdr_error() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::GetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                MockResult::err("GetLinkLevelAddressError")
            });

        let expected_error = "GetLinkLevelAddressError";

        let error = Nic::new().get_lladd(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::SetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                MockResult::ok()
            });

        Nic::new().set_lladd(&IFNAME, &LLADDR).unwrap();
    }

    #[test]
    fn test_set_lladdr_open_error() {
        mockdown().expect(|socket::OpenLocalDgram()| {
            assert!(true);
            OpenSocket::err("SetLinkLevelAddressOpenError")
        });

        let expected_error = "SetLinkLevelAddressOpenError";

        let error = Nic::new().set_lladd(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr_error() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::SetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                MockResult::err("SetLinkLevelAddressError")
            });

        let expected_error = "SetLinkLevelAddressError";

        let error = Nic::new().set_lladd(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }
}
