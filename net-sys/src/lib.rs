#[cfg(not(any(feature = "libc")))]
compile_error!("Unsupported system!");

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("Unsupported target os!");

mod lladdr;

#[cfg_attr(feature = "libc", path = "libc")]
pub mod sys {
    #[cfg_attr(target_os = "linux", path = "linux")]
    #[cfg_attr(target_os = "macos", path = "macos")]
    pub mod os {
        pub mod ifname;
        mod ifreq;
        pub mod nic;
        mod socket;
        mod sys;
    }
}

pub use lladdr::{LLAddr, LinkLevelAddress};
use std::result;
pub use sys::os::{ifname::IfName, nic::Nic};

pub type Result<T> = result::Result<T, Box<dyn std::error::Error>>;
