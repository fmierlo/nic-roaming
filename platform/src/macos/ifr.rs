use std::{ffi::CString, ptr};

use libc::{c_void, ifreq};

pub(crate) fn new() -> ifreq {
    unsafe { std::mem::zeroed() }
}

// .ioctl(s, request, ifr as *mut _ as *mut sys::c_void);
// .ioctl(s, request, ifr as *const _ as *mut sys::c_void);
pub(crate) fn to_c_void_ptr(ifr: &mut ifreq) -> *mut c_void {
    ifr as *mut _ as *mut c_void
}

#[cfg(test)]
pub(crate) fn from_c_void_ptr(arg: *mut c_void) -> &'static mut ifreq {
    unsafe { &mut *(arg as *mut _ as *mut ifreq) }
}

pub(crate) fn get_mac_address(ifr: &ifreq) -> String {
    let mac_address = unsafe { &ifr.ifr_ifru.ifru_addr.sa_data };
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

pub(crate) fn set_mac_address(ifr: &mut ifreq, mac_address: &str) {
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
            ifr.ifr_ifru.ifru_addr.sa_data.as_mut_ptr() as *mut u8,
            6,
        );
    }
    ifr.ifr_ifru.ifru_addr.sa_len = 6;
}

#[cfg(test)]
pub(crate) fn get_name(ifr: &ifreq) -> &str {
    use std::ffi::CStr;

    let ifr_name = unsafe { CStr::from_ptr(ifr.ifr_name.as_ptr()) };
    match ifr_name.to_str() {
        Ok(s) => s,
        Err(_) => "",
    }
}

pub(crate) fn set_name(ifr: &mut ifreq, name: &str) {
    let ifr_name = CString::new(name).unwrap();

    unsafe {
        ptr::copy_nonoverlapping(
            ifr_name.as_ptr(),
            ifr.ifr_name.as_mut_ptr(),
            ifr_name.as_bytes().len(),
        );
    }
}
