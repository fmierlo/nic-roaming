use std::result;

mod ifname;
mod lladdr;

pub use ifname::*;
pub use lladdr::*;

pub type Result<T> = result::Result<T, Box<dyn std::error::Error>>;

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
