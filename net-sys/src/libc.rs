#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("Unsupported target os!");

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::{ifname, ifreq, nic};
