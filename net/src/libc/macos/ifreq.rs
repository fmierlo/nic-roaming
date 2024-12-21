use super::ifname::IfName;
use crate::LinkLevelAddress;
use libc::{c_void, ifreq};
use std::ptr;

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
    IfName::from(ifreq.ifr_name)
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
