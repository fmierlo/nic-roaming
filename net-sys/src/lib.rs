#[cfg(not(any(feature = "libc")))]
compile_error!("Unsupported system!");

#[cfg(feature = "libc")]
pub mod libc;

mod lladdr;

#[cfg(feature = "libc")]
pub use libc::{nic, IfName};

pub use lladdr::{LLAddr, LinkLevelAddress};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
