use core::fmt::{Debug, Display};
use std::ffi::{CString, NulError};
use std::{ops::Deref, ops::DerefMut, ptr};

const IF_NAME_MIN: libc::size_t = 1;
const IF_NAME_MAX: libc::size_t = libc::IFNAMSIZ;

type IfNameType = [libc::c_char; IF_NAME_MAX];

#[derive(Clone, PartialEq, Eq)]
enum Error {
    TooSmall(String),
    TooLarge(String),
    Nul(String, NulError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooSmall(value) => f
                .debug_struct("IfName::TooSmallError")
                .field("value", value)
                .field("len", &value.len())
                .field("min", &IF_NAME_MIN)
                .finish(),
            Self::TooLarge(value) => f
                .debug_struct("IfName::TooLargeError")
                .field("value", value)
                .field("len", &value.len())
                .field("max", &IF_NAME_MAX)
                .finish(),
            Self::Nul(value, error) => f
                .debug_struct("IfName::NulError")
                .field("value", value)
                .field("error", error)
                .finish(),
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct IfName(IfNameType);

impl IfName {
    fn new() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Deref for IfName {
    type Target = IfNameType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IfName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for IfName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from(self))
    }
}

impl Display for IfName {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", String::from(self))
    }
}

impl<'a> From<&IfName> for String {
    fn from(value: &IfName) -> Self {
        let c_str = unsafe { std::ffi::CStr::from_ptr(value.as_ptr()) };
        c_str.to_bytes().escape_ascii().to_string()
    }
}

impl From<IfNameType> for IfName {
    fn from(value: IfNameType) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for IfName {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        String::from(value).try_into()
    }
}

impl TryFrom<String> for IfName {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = match value.len() {
            len if len < IF_NAME_MIN => return Err(Error::TooSmall(value).into()),
            len if len > IF_NAME_MAX => return Err(Error::TooLarge(value).into()),
            _ => CString::new(value.clone()).map_err(|error| Error::Nul(value, error))?,
        };

        let mut ifname = IfName::new();

        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), ifname.as_mut_ptr(), value.as_bytes().len());
        }

        Ok(ifname)
    }
}
