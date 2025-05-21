#[cfg(not(any(feature = "libc")))]
compile_error!("Unsupported system!");

#[cfg(feature = "libc")]
pub mod libc;

pub mod lladdr;

#[cfg(feature = "libc")]
pub use libc::{ifname, nic};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) mod format;
