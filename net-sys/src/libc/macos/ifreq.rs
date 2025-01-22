use std::ptr;

use libc::{c_void, ifreq};

use crate::{IfName, LinkLevelAddress};

pub(super) fn new() -> ifreq {
    unsafe { std::mem::zeroed() }
}

pub(super) fn as_mut_ptr(ifreq: &mut ifreq) -> *mut c_void {
    ifreq as *const _ as *mut c_void
}

pub(super) fn from_mut_ptr<'a>(arg: *mut c_void) -> &'a mut ifreq {
    unsafe { &mut *(arg as *mut _ as *mut ifreq) }
}

pub(super) fn set_name(ifreq: &mut ifreq, ifname: &IfName) {
    unsafe {
        ptr::copy_nonoverlapping(ifname.as_ptr(), ifreq.ifr_name.as_mut_ptr(), ifname.len());
    }
}

pub(super) fn get_name(ifreq: &ifreq) -> IfName {
    IfName::from(&ifreq.ifr_name)
}

pub(super) fn set_lladdr(ifreq: &mut ifreq, lladdr: &LinkLevelAddress) {
    unsafe {
        ptr::copy_nonoverlapping(
            lladdr.as_ptr(),
            ifreq.ifr_ifru.ifru_addr.sa_data.as_mut_ptr() as *mut u8,
            lladdr.len(),
        );
    }
}

pub(super) fn get_lladdr(ifreq: &ifreq) -> LinkLevelAddress {
    let sa_data = unsafe { &*(&ifreq.ifr_ifru.ifru_addr.sa_data as *const _ as *const [u8; 6]) };
    LinkLevelAddress::from(sa_data)
}

#[cfg(test)]
mod tests {
    use libc::{c_char, c_void};

    use crate::{IfName, LinkLevelAddress};

    use super::{as_mut_ptr, from_mut_ptr, get_lladdr, get_name, new, set_lladdr, set_name};

    const IFREQ_SIZE: usize = 32;
    const NAME_SIZE: usize = 16;
    const NAME: [c_char; NAME_SIZE] = [
        // '0'..'9' and 'A'..'F'
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x41, 0x42, 0x43, 0x44, 0x45,
        0x00,
    ];
    const LADDR_SIZE: usize = 6;
    const LLADDR: [u8; LADDR_SIZE] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    struct IfReq<'a>(&'a libc::ifreq);

    impl<'a> std::fmt::Debug for IfReq<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe {
                f.debug_struct("ifreq")
                    .field("ifr_name", &self.0.ifr_name)
                    .field(
                        "ifr_ifru.ifru_addr.sa_len",
                        &self.0.ifr_ifru.ifru_addr.sa_len,
                    )
                    .field(
                        "ifr_ifru.ifru_addr.sa_family",
                        &self.0.ifr_ifru.ifru_addr.sa_family,
                    )
                    .field(
                        "ifr_ifru.ifru_addr.sa_data",
                        &self.0.ifr_ifru.ifru_addr.sa_data,
                    )
                    .finish()
            }
        }
    }

    impl<'a> PartialEq for IfReq<'a> {
        fn eq(&self, other: &Self) -> bool {
            let self_ptr = self.0 as *const _ as *const c_char;
            let other_ptr = other.0 as *const _ as *const c_char;
            for i in 0..IFREQ_SIZE {
                unsafe {
                    if *self_ptr.add(i) != *other_ptr.add(i) {
                        return false;
                    }
                }
            }
            return true;
        }
    }

    #[test]
    fn test_ifreq_size() {
        let expected_size = std::mem::size_of::<libc::ifreq>();

        assert_eq!(IFREQ_SIZE, expected_size);
    }

    #[test]
    fn test_ifreq_new() {
        let expected_ifreq = unsafe { std::mem::zeroed() };

        let ifreq = new();

        assert_eq!(IfReq(&ifreq), IfReq(&expected_ifreq));
    }

    #[test]
    fn test_ifreq_as_mut_ptr() {
        let mut ifreq = new();
        let exptected_ifreq_ptr = &ifreq as *const _ as *mut c_void;

        let ifreq_ptr = as_mut_ptr(&mut ifreq);

        assert_eq!(ifreq_ptr, exptected_ifreq_ptr);
    }

    #[test]
    fn test_ifreq_from_mut_ptr() {
        let mut expected_ifreq = new();
        let ifreq_ptr = &expected_ifreq as *const _ as *mut c_void;

        let ifreq = from_mut_ptr(ifreq_ptr);

        assert_eq!(IfReq(ifreq), IfReq(&mut expected_ifreq));
    }

    #[test]
    fn test_ifreq_set_name() {
        let mut ifreq = new();

        set_name(&mut ifreq, &IfName::from(&NAME));

        assert_eq!(ifreq.ifr_name, NAME);
    }

    #[test]
    fn test_ifreq_get_name() {
        let mut ifreq = new();
        unsafe {
            std::ptr::copy_nonoverlapping(NAME.as_ptr(), ifreq.ifr_name.as_mut_ptr(), NAME.len());
        }

        let ifname = get_name(&mut ifreq);

        assert_eq!(*ifname, NAME);
    }

    #[test]
    fn test_ifreq_set_lladdr() {
        let mut ifreq = new();
        let sa_data_ptr =
            unsafe { &*(&ifreq.ifr_ifru.ifru_addr.sa_data as *const _ as *const [u8; 6]) };

        set_lladdr(&mut ifreq, &LinkLevelAddress::from(&LLADDR));

        assert_eq!(*sa_data_ptr, LLADDR);
    }

    #[test]
    fn test_ifreq_get_lladdr() {
        let mut ifreq = new();
        unsafe {
            std::ptr::copy_nonoverlapping(
                LLADDR.as_ptr(),
                ifreq.ifr_ifru.ifru_addr.sa_data.as_mut_ptr() as *mut u8,
                LLADDR.len(),
            );
        }

        let lladdr = get_lladdr(&mut ifreq);

        assert_eq!(*lladdr, LLADDR);
    }
}

#[cfg(test)]
pub(super) mod mock {
    use crate::{IfName, LinkLevelAddress};

    use super::{from_mut_ptr, get_lladdr, get_name, set_lladdr};

    pub(crate) fn ifreq_get_name(arg: *mut libc::c_void) -> IfName {
        let ifreq = from_mut_ptr(arg);
        get_name(ifreq)
    }

    pub(crate) fn ifreq_get_lladdr(arg: *mut libc::c_void) -> LinkLevelAddress {
        let ifreq = from_mut_ptr(arg);
        get_lladdr(ifreq)
    }

    pub(crate) fn ifreq_set_lladdr(arg: *mut libc::c_void, lladdr: LinkLevelAddress) {
        let ifreq = from_mut_ptr(arg);
        set_lladdr(ifreq, &lladdr);
    }
}
