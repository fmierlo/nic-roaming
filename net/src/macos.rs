mod macos {
    pub mod ifname;
    mod ifreq;
    pub mod nic;
    mod socket;
    mod sys;
}

pub use macos::ifname::IfName;
pub use macos::nic::Nic;
