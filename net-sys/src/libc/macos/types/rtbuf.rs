use libc::c_char;

use super::{ifmamsghdr::IfMaMsgHdr, rtmsghdr::RtMsgHdr};

const RT_BUF_SIZE: usize = 2048;

pub(crate) type RtBuf = [c_char; RT_BUF_SIZE];

pub fn new() -> RtBuf {
    [0; RT_BUF_SIZE]
}

pub(crate) trait AsMsgHdr {
    fn as_rt_msghdr(&self) -> RtMsgHdr;
    fn as_ifma_msghdr(&self) -> IfMaMsgHdr;
}

impl AsMsgHdr for RtBuf {
    fn as_rt_msghdr(&self) -> RtMsgHdr {
        RtMsgHdr(self)
    }

    fn as_ifma_msghdr(&self) -> IfMaMsgHdr {
        IfMaMsgHdr(self)
    }
}
