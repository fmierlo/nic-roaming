use std::{error::Error, ffi::c_char, result};

mod ifname;
mod lladdr;

pub use ifname::*;
pub use lladdr::*;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

fn str_from_ptr<'a>(ptr: *const c_char) -> result::Result<&'a str, std::str::Utf8Error> {
    let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
    c_str.to_str()
}

fn str_from_ptr_or_empty<'a>(ptr: *const c_char) -> &'a str {
    str_from_ptr(ptr).unwrap_or("")
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("Unsupported platform!");

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use linux::*;
