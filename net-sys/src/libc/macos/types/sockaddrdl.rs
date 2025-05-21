use core::fmt::Debug;
use std::ops::Deref;

use libc::{c_char, c_int, c_ushort};

use crate::ifname::IfName;
use crate::lladdr::LinkLevelAddress;

use super::super::defs::af::Af;
use super::super::defs::ift::Ift;

trait SockaddrDl {
    fn sdl_family(&self) -> Af;
    fn sdl_type(&self) -> Ift;
    fn get_data(&self) -> (&[c_char], &[c_char], &[c_char]);
}

impl SockaddrDl for libc::sockaddr_dl {
    fn sdl_family(&self) -> Af {
        Af::from(self.sdl_family as c_int)
    }

    fn sdl_type(&self) -> Ift {
        Ift::from(self.sdl_type as c_int)
    }

    fn get_data(&self) -> (&[c_char], &[c_char], &[c_char]) {
        let data = &self.sdl_data;
        let (name, data) = data.split_at(self.sdl_nlen as usize); // Name
        let (addr, data) = data.split_at(self.sdl_alen as usize); // Address
        let (sel, _data) = data.split_at(self.sdl_slen as usize); // Selector
        (name, addr, sel)
    }
}

pub(crate) trait LinkEther {
    fn get_link_ether(&self) -> Option<(c_ushort, IfName, LinkLevelAddress)>;
}

impl LinkEther for libc::sockaddr_dl {
    fn get_link_ether(&self) -> Option<(c_ushort, IfName, LinkLevelAddress)> {
        if self.sdl_family() != Af::AfLink || self.sdl_type() != Ift::IftEther {
            return None;
        }

        let (name, addr, _sel) = self.get_data();

        let ifname = match IfName::try_from(name) {
            Ok(ifname) => ifname,
            Err(_) => match IfName::try_from(format!("index{}", self.sdl_index)) {
                Ok(ifname) => ifname,
                Err(_) => return None,
            },
        };

        let lladdr = match LinkLevelAddress::try_from(addr) {
            Ok(lladdr) => lladdr,
            Err(_) => return None,
        };

        let (_, _) = (name, addr);

        Some((self.sdl_index, ifname, lladdr))
    }
}

struct SockaddrDlDebug<'a>(&'a libc::sockaddr_dl);

impl<'a> Deref for SockaddrDlDebug<'a> {
    type Target = &'a libc::sockaddr_dl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Debug for SockaddrDlDebug<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sockaddr_dl {{")?;
        write!(f, "sdl_len: {:?}, ", &self.sdl_len)?;
        write!(f, "sdl_family: {:?}, ", &self.sdl_family())?;
        write!(f, "sdl_index: {:?}, ", &self.sdl_index)?;
        write!(f, "sdl_type: {:?}, ", &self.sdl_type())?;
        write!(f, "sdl_nlen: {:?}, ", &self.sdl_nlen)?;
        write!(f, "sdl_alen: {:?}, ", &self.sdl_alen)?;
        write!(f, "sdl_slen: {:?}, ", &self.sdl_slen)?;

        match self.get_link_ether() {
            Some(nic) => {
                let (_, ifname, lladdr) = nic;
                write!(f, "sdl_ifname: {:?}, ", ifname)?;
                write!(f, "sdl_lladdr: {:?}, ", lladdr)?;
            }
            None => (),
        }

        write!(f, "sdl_data: {:?} }}", &self.sdl_data)?;
        Ok(())
    }
}
