use core::fmt::{Debug, Display};
use std::ops::Deref;
use std::mem;

use libc::c_int;

use crate::format::AsBytes;

use super::{rtbuf::RtBuf};
use super::super::defs::rtm::Rtm;

pub(crate) struct RtMsgHdr<'a>(pub(crate) &'a RtBuf);

impl<'a> Deref for RtMsgHdr<'a> {
    type Target = libc::rt_msghdr;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self.0) }
    }
}

impl<'a> RtMsgHdr<'a> {
    pub fn rtm_type(&self) -> Rtm {
        Rtm::from(self.rtm_type as c_int)
    }
}

impl<'a> Display for RtMsgHdr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("rt_msghdr")
            .field("rtm_msglen", &self.rtm_msglen)
            .field("rtm_version", &self.rtm_version)
            .field("rtm_type", &self.rtm_type())
            .finish_non_exhaustive()
    }
}

impl<'a> Debug for RtMsgHdr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("rt_msghdr")
            .field("rtm_msglen", &self.rtm_msglen)
            .field("rtm_version", &self.rtm_version)
            .field("rtm_type", &self.rtm_type())
            .field("rtm_index", &self.rtm_index)
            .field("rtm_flags", &self.rtm_flags)
            .field("rtm_addrs", &self.rtm_addrs)
            .field("rtm_pid", &self.rtm_pid)
            .field("rtm_seq", &self.rtm_seq)
            .field("rtm_errno", &self.rtm_errno)
            .field("rtm_use", &self.rtm_use)
            .field("rtm_inits", &self.rtm_inits)
            .field("rtm_rmx", &self.rtm_rmx.as_lower_hex())
            .finish()
    }
}
