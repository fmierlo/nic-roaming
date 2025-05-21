use std::{mem, ptr};

use libc::c_void;

use crate::ifname::IfName;
use crate::lladdr::{LinkLevelAddress, SignedOctetsType};

pub(crate) fn new() -> libc::ifreq {
    unsafe { std::mem::zeroed() }
}

pub(crate) trait IfReqWith {
    fn with_name(self, ifname: &IfName) -> Self;
    fn with_lladdr(self, lladdr: &LinkLevelAddress) -> Self;
}

impl IfReqWith for libc::ifreq {
    fn with_name(mut self, ifname: &IfName) -> Self {
        self.change_name(ifname);
        self
    }

    fn with_lladdr(mut self, lladdr: &LinkLevelAddress) -> Self {
        self.change_lladdr(lladdr);
        self
    }
}

pub(crate) trait IfReqMut {
    fn change_name(&mut self, ifname: &IfName);
    fn change_lladdr(&mut self, lladdr: &LinkLevelAddress);
}

impl IfReqMut for libc::ifreq {
    fn change_name(&mut self, ifname: &IfName) {
        unsafe {
            ptr::copy_nonoverlapping(ifname.as_ptr(), self.ifr_name.as_mut_ptr(), ifname.len());
        }
    }

    fn change_lladdr(&mut self, lladdr: &LinkLevelAddress) {
        unsafe {
            ptr::copy_nonoverlapping(
                lladdr.as_signed_ptr(),
                self.ifr_ifru.ifru_addr.sa_data.as_mut_ptr(),
                lladdr.len(),
            );
        }
        self.ifr_ifru.ifru_addr.sa_len = lladdr.len() as u8;
    }
}

pub(crate) trait IfReq {
    fn name(&self) -> IfName;
    fn lladdr(&self) -> LinkLevelAddress;
}

impl IfReq for libc::ifreq {
    fn name(&self) -> IfName {
        IfName::from(&self.ifr_name)
    }

    fn lladdr(&self) -> LinkLevelAddress {
        let sa_data = unsafe { &self.ifr_ifru.ifru_addr.sa_data };
        let sa_data: &SignedOctetsType = unsafe { mem::transmute(sa_data) };
        LinkLevelAddress::from(sa_data)
    }
}

pub(crate) trait IfReqAsPtr {
    fn as_mut_ptr(&mut self) -> *mut c_void;
}

impl IfReqAsPtr for libc::ifreq {
    fn as_mut_ptr(&mut self) -> *mut c_void {
        unsafe { mem::transmute(self) }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::mem;

    use libc::{c_char, c_void};

    use crate::format::AsBytes;
    use crate::ifname::IfName;
    use crate::lladdr::{LinkLevelAddress, SignedOctetsType};
    use crate::Result;

    use super::new;
    use super::{IfReq, IfReqAsPtr, IfReqMut, IfReqWith};

    const IFREQ_SIZE: usize = 32;
    const NAME_SIZE: usize = 16;
    const NAME: [c_char; NAME_SIZE] = [
        // '0'..'9' and 'A'..'F'
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x41, 0x42, 0x43, 0x44, 0x45,
        0x00,
    ];
    const LLADDR_SIZE: usize = 6;
    const LLADDR: [c_char; LLADDR_SIZE] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    pub(crate) trait PtrAsIfReq {
        fn as_ifreq<'a>(&self) -> &'a mut libc::ifreq;
    }

    impl PtrAsIfReq for *mut c_void {
        fn as_ifreq<'a>(&self) -> &'a mut libc::ifreq {
            unsafe { mem::transmute(*self) }
        }
    }

    impl PtrAsIfReq for *mut libc::ifreq {
        fn as_ifreq<'a>(&self) -> &'a mut libc::ifreq {
            unsafe { mem::transmute(*self) }
        }
    }

    #[test]
    fn test_ifreq_size() {
        let expected_size = size_of::<libc::ifreq>();

        assert_eq!(IFREQ_SIZE, expected_size);
    }

    #[test]
    fn test_ifreq_new() {
        let expected_ifreq: libc::ifreq = unsafe { std::mem::zeroed() };

        let ifreq = new();

        assert_eq!(ifreq.as_lower_hex(), expected_ifreq.as_lower_hex());
    }

    #[test]
    fn test_ifreq_with_name() {
        let ifreq = new().with_name(&IfName::from(&NAME));

        assert_eq!(ifreq.ifr_name, NAME);
    }

    #[test]
    fn test_ifreq_with_lladdr() -> Result<()> {
        let ifreq = new().with_lladdr(&LinkLevelAddress::from(&LLADDR));

        let sa_data = unsafe { &ifreq.ifr_ifru.ifru_addr.sa_data };
        let sa_data_ref: &SignedOctetsType = unsafe { mem::transmute(sa_data) };

        assert_eq!((*sa_data_ref).as_lower_hex(), LLADDR.as_lower_hex());

        Ok(())
    }

    #[test]
    fn test_ifreq_change_name() {
        let mut ifreq = new();

        ifreq.change_name(&IfName::from(&NAME));

        assert_eq!(ifreq.ifr_name, NAME);
    }

    #[test]
    fn test_ifreq_change_lladdr() -> Result<()> {
        let mut ifreq = new();

        let sa_data = unsafe { &ifreq.ifr_ifru.ifru_addr.sa_data };
        let sa_data_ref: &SignedOctetsType = unsafe { mem::transmute(sa_data) };

        ifreq.change_lladdr(&LinkLevelAddress::from(&LLADDR));

        assert_eq!((*sa_data_ref).as_lower_hex(), LLADDR.as_lower_hex());
        assert_eq!(
            unsafe { ifreq.ifr_ifru.ifru_addr.sa_len },
            LLADDR_SIZE as u8
        );

        Ok(())
    }

    #[test]
    fn test_ifreq_name() {
        let mut ifreq = new();

        unsafe {
            std::ptr::copy_nonoverlapping(NAME.as_ptr(), ifreq.ifr_name.as_mut_ptr(), NAME.len());
        }

        let ifname = ifreq.name();

        assert_eq!(*ifname, NAME);
    }

    #[test]
    fn test_ifreq_lladdr() {
        let mut ifreq = new();
        unsafe {
            std::ptr::copy_nonoverlapping(
                LLADDR.as_ptr(),
                ifreq.ifr_ifru.ifru_addr.sa_data.as_mut_ptr(),
                LLADDR.len(),
            );
        }

        let lladdr = ifreq.lladdr();

        assert_eq!(*lladdr.as_signed_ref(), LLADDR);
    }

    #[test]
    fn test_ifreq_as_mut_ptr() {
        let mut ifreq = new();
        let exptected_ifreq_ptr: *mut c_void = unsafe { mem::transmute(&ifreq) };

        let ifreq_ptr = ifreq.as_mut_ptr();

        assert_eq!(ifreq_ptr, exptected_ifreq_ptr);
    }

    #[test]
    fn test_mut_ptr_as_ifreq() {
        let mut expected_ifreq = new();
        let ifreq_ptr: *mut c_void = unsafe { mem::transmute(&mut expected_ifreq) };

        let ifreq = ifreq_ptr.as_ifreq();

        assert_eq!((*ifreq).as_lower_hex(), expected_ifreq.as_lower_hex());
    }
}
