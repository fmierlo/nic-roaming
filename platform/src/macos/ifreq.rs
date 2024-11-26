use std::{ffi::CString, ptr};

use libc::{c_void, ifreq};

use crate::Result;

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

pub(crate) fn set_mac_address(ifreq: &mut ifreq, mac_address: &str) -> Result<()> {
    let mac_bytes: Vec<u8> = mac_address
        .split(':')
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();

    if mac_bytes.len() != 6 {
        return Err(format!("MAC address isn't 6 bytes in hex format: mac_address={mac_address} len={}", mac_bytes.len()).into());
    }

    unsafe {
        ptr::copy_nonoverlapping(
            mac_bytes.as_ptr(),
            ifreq.ifr_ifru.ifru_addr.sa_data.as_mut_ptr() as *mut u8,
            6,
        );
    }

    ifreq.ifr_ifru.ifru_addr.sa_len = 6;

    Ok(())
}

pub(crate) fn get_mac_address(ifreq: &ifreq) -> String {
    let mac_address = unsafe { &ifreq.ifr_ifru.ifru_addr.sa_data };
    let mac_str = format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac_address[0],
        mac_address[1],
        mac_address[2],
        mac_address[3],
        mac_address[4],
        mac_address[5]
    );
    mac_str
}
