use crate::ifname::IfName;
use crate::ifreq::IfReq;
use crate::lladdr::LinkLevelAddress;
use crate::Result;

use super::ifreq::{self, IfReqWith};
#[cfg(not(test))]
use super::socket;

#[cfg(test)]
use mocks::socket;

pub fn get_lladdr(ifname: &IfName) -> Result<LinkLevelAddress> {
    let mut ifreq = ifreq::new().with_name(ifname);

    socket::open_local_dgram()?.get_lladdr(&mut ifreq)?;

    Ok(ifreq.lladdr())
}

pub fn set_lladdr(ifname: &IfName, lladdr: &LinkLevelAddress) -> Result<()> {
    let mut ifreq = ifreq::new().with_name(ifname).with_lladdr(lladdr);

    socket::open_local_dgram()?.set_lladdr(&mut ifreq)
}

#[cfg(test)]
pub(crate) mod mocks {
    pub(crate) mod socket {
        use mockdown::{mockdown, Mock};

        use crate::Result;

        pub(crate) struct OpenLocalDgram(pub fn() -> Result<OpenSocket>);
        pub(crate) struct GetLLAddr(pub fn(ifreq: &mut libc::ifreq) -> Result<()>);
        pub(crate) struct SetLLAddr(pub fn(ifreq: &mut libc::ifreq) -> Result<()>);

        pub(crate) fn open_local_dgram() -> Result<OpenSocket> {
            mockdown().next(|OpenLocalDgram(mock)| mock())?
        }

        pub(crate) struct OpenSocket();

        impl OpenSocket {
            pub(crate) fn get_lladdr(&self, ifreq: &mut libc::ifreq) -> Result<()> {
                mockdown().next(|GetLLAddr(mock)| mock(ifreq))?
            }

            pub(crate) fn set_lladdr(&self, ifreq: &mut libc::ifreq) -> Result<()> {
                mockdown().next(|SetLLAddr(mock)| mock(ifreq))?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use mockdown::{mockdown, Mock};

    use crate::ifname::IfName;
    use crate::ifreq::{IfReq, IfReqMut};
    use crate::lladdr::LinkLevelAddress;
    use crate::Result;

    use super::mocks::socket::{self, OpenSocket};
    use super::{get_lladdr, set_lladdr};

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    #[test]
    fn test_get_lladdr() -> Result<()> {
        mockdown()
            .expect(socket::OpenLocalDgram(|| Ok(OpenSocket())))
            .expect(socket::GetLLAddr(|ifreq| {
                assert_eq!(ifreq.name(), *IFNAME);
                ifreq.change_lladdr(&LLADDR);
                Ok(())
            }));

        let lladdr = get_lladdr(&IFNAME)?;

        assert_eq!(lladdr, *LLADDR);

        Ok(())
    }

    #[test]
    fn test_get_lladdr_open_error() {
        mockdown().expect(socket::OpenLocalDgram(|| {
            Err("GetLinkLevelAddressOpenError".into())
        }));

        let expected_error = "GetLinkLevelAddressOpenError";

        let error = get_lladdr(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_get_lladdr_error() {
        mockdown()
            .expect(socket::OpenLocalDgram(|| Ok(OpenSocket())))
            .expect(socket::GetLLAddr(|ifreq| {
                assert_eq!(ifreq.name(), *IFNAME);
                Err("GetLinkLevelAddressError".into())
            }));

        let expected_error = "GetLinkLevelAddressError";

        let error = get_lladdr(&IFNAME).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr() -> Result<()> {
        mockdown()
            .expect(socket::OpenLocalDgram(|| Ok(OpenSocket())))
            .expect(socket::SetLLAddr(|ifreq| {
                assert_eq!(ifreq.name(), *IFNAME);
                assert_eq!(ifreq.lladdr(), *LLADDR);
                Ok(())
            }));

        set_lladdr(&IFNAME, &LLADDR)
    }

    #[test]
    fn test_set_lladdr_open_error() {
        mockdown().expect(socket::OpenLocalDgram(|| {
            Err("SetLinkLevelAddressOpenError".into())
        }));

        let expected_error = "SetLinkLevelAddressOpenError";

        let error = set_lladdr(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }

    #[test]
    fn test_set_lladdr_error() {
        mockdown()
            .expect(socket::OpenLocalDgram(|| Ok(OpenSocket())))
            .expect(socket::SetLLAddr(|ifreq| {
                assert_eq!(ifreq.name(), *IFNAME);
                assert_eq!(ifreq.lladdr(), *LLADDR);
                Err("SetLinkLevelAddressError".into())
            }));

        let expected_error = "SetLinkLevelAddressError";

        let error = set_lladdr(&IFNAME, &LLADDR).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
    }
}
