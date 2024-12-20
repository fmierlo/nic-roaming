#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("Unsupported platform!");

mod ifname;
mod lladdr;
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "macos", path = "macos.rs")]
mod os;

pub use ifname::IfName;
pub use lladdr::{LLAddr, LinkLevelAddress};
pub use os::Nic;
use std::result;

pub type Result<T> = result::Result<T, Box<dyn std::error::Error>>;
