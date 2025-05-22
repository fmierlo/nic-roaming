#[cfg(not(any(feature = "libc")))]
compile_error!("Unsupported system!");

pub(crate) mod format;
pub mod ifname;
pub mod lladdr;

#[cfg(feature = "libc")]
mod libc;

#[cfg(feature = "libc")]
pub use libc::{nic, IF_NAME_SIZE};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
