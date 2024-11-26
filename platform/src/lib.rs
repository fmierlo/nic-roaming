use std::{error::Error, result};

pub type Result<T> = result::Result<T, Box<dyn Error>>;

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
