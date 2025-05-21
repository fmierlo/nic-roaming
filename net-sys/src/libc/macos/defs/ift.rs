use std::fmt::Debug;

use libc::c_int;

// https://github.com/apple/darwin-xnu/blob/xnu-7195.121.3/bsd/net/if_types.h#L81

const IFT_ETHER: c_int = 0x06;
const IFT_LOOP: c_int = 0x18;

// Interface Types
#[repr(i32)]
#[derive(PartialEq)]
pub(crate) enum Ift {
    IftEther = IFT_ETHER,
    IftLoop = IFT_LOOP,
    IftInvalid(c_int),
}

impl From<c_int> for Ift {
    fn from(value: c_int) -> Self {
        match value {
            IFT_ETHER => Ift::IftEther,
            IFT_LOOP => Ift::IftLoop,
            value => Ift::IftInvalid(value),
        }
    }
}

impl Debug for Ift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IftEther => write!(f, "IftEther"),
            Self::IftLoop => write!(f, "IftLoop"),
            Self::IftInvalid(value) => f
                .debug_tuple("IftInvalid")
                .field(&format!("{:x}", value))
                .finish(),
        }
    }
}
