use std::{ffi::CString, ptr};

use libc::{c_void, ifreq};

use crate::{LinkLevelAddress, Result};

pub(crate) fn new() -> ifreq {
    unsafe { std::mem::zeroed() }
}

pub(crate) fn as_mut_ptr(ifreq: &mut ifreq) -> *mut c_void {
    ifreq as *const _ as *mut c_void
}

#[cfg(test)]
pub(crate) fn from_mut_ptr<'a>(arg: *mut c_void) -> &'a mut ifreq {
    unsafe { &mut *(arg as *mut _ as *mut ifreq) }
}

pub(crate) fn set_name(ifreq: &mut ifreq, name: &str) -> Result<()> {
    let name = CString::new(name)?;
    unsafe {
        ptr::copy_nonoverlapping(
            name.as_ptr(),
            ifreq.ifr_name.as_mut_ptr(),
            name.as_bytes().len(),
        );
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn get_name(ifreq: &ifreq) -> Result<String> {
    use std::ffi::CStr;
    let name = unsafe { CStr::from_ptr(ifreq.ifr_name.as_ptr()) };
    let name = name.to_str()?;
    Ok(String::from(name))
}

pub(crate) fn set_lladdr(ifreq: &mut ifreq, lladdr: LinkLevelAddress) -> Result<()> {
    unsafe {
        ptr::copy_nonoverlapping(
            lladdr.as_ptr(),
            ifreq.ifr_ifru.ifru_addr.sa_data.as_mut_ptr() as *mut u8,
            lladdr.len(),
        );
    }
    Ok(())
}

pub(crate) fn get_lladdr(ifreq: &ifreq) -> Result<LinkLevelAddress> {
    let sa_data = unsafe { &*(&ifreq.ifr_ifru.ifru_addr.sa_data as *const _ as *const [u8; 6]) };
    Ok(LinkLevelAddress::from(sa_data))
}
