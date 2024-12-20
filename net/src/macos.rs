mod macos {
    mod ifreq;
    pub mod nic;
    mod socket;
    mod sys;
}

pub use macos::nic::Nic;
