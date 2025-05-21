use std::fmt::Debug;

use crate::ifname::IfName;
use crate::lladdr::LinkLevelAddress;
use crate::Result;

use super::defs::rtm::Rtm;
use super::types::ifreq::{self, IfReq, IfReqWith};
use super::types::rtbuf::{self, AsMsgHdr, RtBuf};
use super::types::sockaddrdl::LinkEther;

#[cfg(not(test))]
use super::socket;
use libc::c_ushort;
#[cfg(test)]
use mocks::socket;

use super::socket::ReadResult::{EndOfRead, ReadLength};

#[derive(Clone, Debug)]
pub enum NicEvent {
    NicNew((c_ushort, IfName, LinkLevelAddress)),
    NicDel((c_ushort, IfName, LinkLevelAddress)),
    NicNoop,
}

pub fn monitor() -> Result<NicMonitor> {
    Ok(NicMonitor {
        socket: socket::open_route_raw()?,
    })
}

#[derive(Debug)]
pub struct NicMonitor {
    socket: socket::OpenSocket,
}

// Source: https://github.com/freebsd/freebsd-src/blob/main/sbin/route/route.c

impl NicMonitor {
    fn parse_msg(&self, rt_buf: &RtBuf, _len: isize) -> Option<NicEvent> {
        let rtm = rt_buf.as_rt_msghdr();

        if rtm.rtm_version as i32 != libc::RTM_VERSION {
            eprintln!(
                "routing message version {} is not understood",
                rtm.rtm_version
            );
            return Some(NicEvent::NicNoop);
        }

        let event = match rtm.rtm_type() {
            Rtm::RtmNewmaddr => {
                let nic = rt_buf.as_ifma_msghdr().get_ifp()?.get_link_ether()?;
                NicEvent::NicNew(nic)
            }
            Rtm::RtmDelmaddr => {
                let nic = rt_buf.as_ifma_msghdr().get_ifp()?.get_link_ether()?;
                NicEvent::NicDel(nic)
            }
            Rtm::RtmInvalid(value) => {
                eprintln!("{:?}", Rtm::RtmInvalid(value));
                NicEvent::NicNoop
            }
            _ => NicEvent::NicNoop,
        };

        Some(event)
    }
}

impl Iterator for NicMonitor {
    type Item = Result<NicEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rt_buf = rtbuf::new();
        let event = match self.socket.read(&mut rt_buf) {
            Ok(ReadLength(len)) => match self.parse_msg(&rt_buf, len) {
                Some(event) => Ok(event),
                None => Ok(NicEvent::NicNoop),
            },
            Ok(EndOfRead) => return None,
            Err(err) => Err(err),
        };

        Some(event)
    }
}

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
        use libc::c_char;
        use mockdown::{mockdown, Mock};

        use crate::libc::macos::socket::ReadResult;
        use crate::Result;

        pub(crate) struct OpenLocalDgram(pub fn() -> Result<OpenSocket>);
        pub(crate) struct OpenRouteRaw(pub fn() -> Result<OpenSocket>);
        pub(crate) struct GetLLAddr(pub fn(ifreq: &mut libc::ifreq) -> Result<()>);
        pub(crate) struct SetLLAddr(pub fn(ifreq: &mut libc::ifreq) -> Result<()>);
        pub(crate) struct Read(pub fn(buf: &mut [c_char]) -> Result<ReadResult>);

        pub(crate) fn open_local_dgram() -> Result<OpenSocket> {
            mockdown().next(|OpenLocalDgram(mock)| mock())?
        }

        pub(crate) fn open_route_raw() -> Result<OpenSocket> {
            mockdown().next(|OpenRouteRaw(mock)| mock())?
        }

        #[derive(Debug)]
        pub(crate) struct OpenSocket();

        impl OpenSocket {
            pub(crate) fn get_lladdr(&self, ifreq: &mut libc::ifreq) -> Result<()> {
                mockdown().next(|GetLLAddr(mock)| mock(ifreq))?
            }

            pub(crate) fn set_lladdr(&self, ifreq: &mut libc::ifreq) -> Result<()> {
                mockdown().next(|SetLLAddr(mock)| mock(ifreq))?
            }
            pub(crate) fn read(&self, buf: &mut [c_char]) -> Result<ReadResult> {
                mockdown().next(|Read(mock)| mock(buf))?
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use mockdown::{mockdown, Mock};

    use crate::ifname::IfName;
    use crate::lladdr::LinkLevelAddress;
    use crate::Result;

    use super::super::types::ifreq::{IfReq, IfReqMut};
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
