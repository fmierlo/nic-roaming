use core::fmt::Debug;
use std::ops::Deref;
use std::mem;

use libc::{c_char, c_int};

use super::super::defs::rtm::Rtm;
use super::rtbuf::RtBuf;

pub(crate) struct IfMaMsgHdr<'a>(pub(crate) &'a RtBuf);

impl<'a> Deref for IfMaMsgHdr<'a> {
    type Target = libc::ifma_msghdr;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self.0) }
    }
}

impl<'a> IfMaMsgHdr<'a> {
    pub fn ifmam_type(&self) -> Rtm {
        Rtm::from(self.ifmam_type as c_int)
    }

    /// Maps `rta` to its corresponding index in the ifma_msg's address array,
    /// if it is set in `ifmam_addrs` bitmask.
    ///
    /// # Returns
    /// `Some(index)` if `rta` is set in `ifmam_addrs`, `None` otherwise.
    fn get_rta_index(&self, rta: c_int) -> Option<usize> {
        (libc::RTAX_DST..libc::RTAX_MAX)
            .map(|rtax| 1 << rtax)
            .filter(|bitmask| self.ifmam_addrs & *bitmask != 0)
            .position(|bitmask| rta & bitmask != 0)
    }

    const HDR_SIZE: usize = size_of::<libc::ifma_msghdr>();
    const SDL_SIZE: usize = size_of::<libc::sockaddr_dl>();

    fn get_rta_buf(&self, rta: c_int) -> Option<&[c_char]> {
        let index = self.get_rta_index(rta)?;
        let data = &(self.0);
        let (_hdr, data) = data.split_at(Self::HDR_SIZE);
        let (_start, data) = data.split_at(Self::SDL_SIZE * index);
        let (buf, _end) = data.split_at(Self::SDL_SIZE);
        Some(buf)
    }

    pub fn get_ifp(&self) -> Option<&libc::sockaddr_dl> {
        let buf = self.get_rta_buf(libc::RTA_IFP)?;
        Some(unsafe { mem::transmute(buf.as_ptr()) })
    }
}

impl<'a> Debug for IfMaMsgHdr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ifma_msghdr")
            .field("ifmam_msglen", &self.ifmam_msglen)
            .field("ifmam_version", &self.ifmam_version)
            .field("ifmam_type", &self.ifmam_type())
            .field("ifmam_addrs", &format!("0x{:x}", &self.ifmam_addrs))
            .field("ifmam_flags", &format!("0x{:x}", &self.ifmam_flags))
            .field("ifmam_index", &self.ifmam_index)
            .finish()
    }
}
