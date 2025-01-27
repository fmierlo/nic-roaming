use crate::ifname::IfName;
use crate::ifreq::IfReq;
use crate::lladdr::LinkLevelAddress;
use crate::Result;

use super::ifreq::{self, IfReqAsPtr, IfReqWith};
#[cfg(not(test))]
use super::socket;

#[cfg(test)]
use mocks::socket;

pub fn get_lladdr(ifname: &IfName) -> Result<LinkLevelAddress> {
    let mut ifreq = ifreq::new().with_name(ifname);

    socket::open_local_dgram()?.get_lladdr(ifreq.as_mut_ptr())?;

    Ok(ifreq.lladdr())
}

pub fn set_lladdr(ifname: &IfName, lladdr: &LinkLevelAddress) -> Result<()> {
    let mut ifreq = ifreq::new().with_name(ifname).with_lladdr(lladdr);

    socket::open_local_dgram()?.set_lladdr(ifreq.as_mut_ptr())
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

            pub(crate) fn get_lladdr(&self, ifreq_ptr: *mut libc::c_void) -> Result<()> {
                let args = GetLLAddr(ifreq_ptr);
                mockdown().mock(args)?
            }

            pub(crate) fn set_lladdr(&self, ifreq_ptr: *mut libc::c_void) -> Result<()> {
                let args = SetLLAddr(ifreq_ptr);
                mockdown().mock(args)?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use mockdown::{mockdown, Mock};

    use crate::ifname::IfName;
    use crate::ifreq::{IfReq, IfReqMut, PtrAsIfReq};
    use crate::lladdr::LinkLevelAddress;

    use super::mocks::socket::{self, MockResult, OpenSocket};
    use super::{get_lladdr, set_lladdr};

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
            .expect(|socket::GetLLAddr(ifreq_ptr)| {
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                ifreq_ptr.as_ifreq().change_lladdr(&LLADDR);
                MockResult::ok()
            });

        let lladdr = get_lladdr(&IFNAME).unwrap();

        assert_eq!(lladdr, *LLADDR);
    }

    #[test]
    fn test_get_lladdr_open_error() {
        mockdown().expect(|socket::OpenLocalDgram()| {
            assert!(true);
            OpenSocket::err("GetLinkLevelAddressOpenError")
        });

        let expected_error = "GetLinkLevelAddressOpenError";

        let error = get_lladdr(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_get_lladdr_error() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::GetLLAddr(ifreq_ptr)| {
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                MockResult::err("GetLinkLevelAddressError")
            });

        let expected_error = "GetLinkLevelAddressError";

        let error = get_lladdr(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::SetLLAddr(ifreq_ptr)| {
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                assert_eq!(ifreq_ptr.as_ifreq().lladdr(), *LLADDR);
                MockResult::ok()
            });

        set_lladdr(&IFNAME, &LLADDR).unwrap();
    }

    #[test]
    fn test_set_lladdr_open_error() {
        mockdown().expect(|socket::OpenLocalDgram()| {
            assert!(true);
            OpenSocket::err("SetLinkLevelAddressOpenError")
        });

        let expected_error = "SetLinkLevelAddressOpenError";

        let error = set_lladdr(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr_error() {
        mockdown()
            .expect(|socket::OpenLocalDgram()| {
                assert!(true);
                OpenSocket::ok()
            })
            .expect(|socket::SetLLAddr(ifreq_ptr)| {
                assert_eq!(ifreq_ptr.as_ifreq().name(), *IFNAME);
                assert_eq!(ifreq_ptr.as_ifreq().lladdr(), *LLADDR);
                MockResult::err("SetLinkLevelAddressError")
            });

        let expected_error = "SetLinkLevelAddressError";

        let error = set_lladdr(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }
}
