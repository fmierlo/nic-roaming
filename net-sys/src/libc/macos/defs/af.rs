use std::fmt::Debug;

use libc::c_int;

// https://github.com/apple/darwin-xnu/blob/xnu-7195.121.3/bsd/sys/socket.h#L540

// Address Family
#[repr(i32)]
#[derive(PartialEq)]
pub(crate) enum Af {
    AfInet = libc::AF_INET,
    AfInet6 = libc::AF_INET6,
    AfLink = libc::AF_LINK,
    AfInvalid(c_int),
}

impl From<c_int> for Af {
    fn from(value: c_int) -> Self {
        match value {
            libc::AF_INET => Af::AfInet,
            libc::AF_INET6 => Af::AfInet6,
            libc::AF_LINK => Af::AfLink,
            value => Af::AfInvalid(value),
        }
    }
}

impl Debug for Af {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AfInet => write!(f, "AfInet"),
            Self::AfInet6 => write!(f, "AfInet6"),
            Self::AfLink => write!(f, "AfLink"),
            Self::AfInvalid(value) => f
                .debug_tuple("AfInvalid")
                .field(&format!("{:x}", value))
                .finish(),
        }
    }
}
