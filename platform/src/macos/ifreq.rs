use std::{ffi::CString, ptr};

use libc::{c_void, ifreq};

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

pub(crate) fn set_name(ifreq: &mut ifreq, name: &str) {
    let name = CString::new(name).unwrap();

    unsafe {
        ptr::copy_nonoverlapping(
            name.as_ptr(),
            ifreq.ifr_name.as_mut_ptr(),
            name.as_bytes().len(),
        );
    }
}

#[cfg(test)]
pub(crate) fn get_name(ifreq: &ifreq) -> String {
    use std::ffi::CStr;

    let name = unsafe { CStr::from_ptr(ifreq.ifr_name.as_ptr()) };
    match name.to_str() {
        Ok(s) => String::from(s),
        Err(_) => String::from(""),
    }
}

pub(crate) fn set_mac_address(ifreq: &mut ifreq, mac_address: &str) {
    let mac_bytes: Vec<u8> = mac_address
        .split(':')
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect();
    // if mac_bytes.len() != 6 {
    //     eprintln!("ERROR: Invalid MAC address format. Must be 6 bytes in hex format.");
    //     return ControlFlow::Break(());
    // }
    unsafe {
        // Copy MAC address into sockaddr_dl
        ptr::copy_nonoverlapping(
            mac_bytes.as_ptr(),
            ifreq.ifr_ifru.ifru_addr.sa_data.as_mut_ptr() as *mut u8,
            6,
        );
    }
    ifreq.ifr_ifru.ifru_addr.sa_len = 6;
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
