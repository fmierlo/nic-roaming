use std::{
    error::Error,
    ffi::{CString, NulError},
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr,
};

const IF_NAME_MIN: libc::size_t = 1;
const IF_NAME_MAX: libc::size_t = libc::IFNAMSIZ;

type IfNameType = [libc::c_char; IF_NAME_MAX];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind<'a> {
    TooSmall(&'a str),
    TooLarge(&'a str),
    NulError(&'a str, NulError),
}

impl<'a> Display for ErrorKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::TooSmall(value) => {
                let len = value.len();
                write!(f, "`{value}` len {len} is too small, min {IF_NAME_MIN}")
            }
            ErrorKind::TooLarge(value) => {
                let len = value.len();
                write!(f, "`{value}` len {len} is too large, max {IF_NAME_MAX}")
            }
            ErrorKind::NulError(value, error) => {
                write!(f, "error converting value `{value}` to CString: {error}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfNameError(String);

impl<'a> From<ErrorKind<'a>> for IfNameError {
    fn from(value: ErrorKind) -> Self {
        Self(value.to_string())
    }
}

impl Display for IfNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failure converting to IfName, {}", self.0)
    }
}

impl Error for IfNameError {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct IfName {
    ifname: IfNameType,
}

impl IfName {
    fn new() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl Deref for IfName {
    type Target = IfNameType;

    fn deref(&self) -> &Self::Target {
        &self.ifname
    }
}

impl DerefMut for IfName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ifname
    }
}

impl Display for IfName {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c_str = unsafe { std::ffi::CStr::from_ptr(self.as_ptr()) };
        write!(fmt, "{}", c_str.to_bytes().escape_ascii().to_string())
    }
}

impl From<IfNameType> for IfName {
    fn from(value: IfNameType) -> Self {
        Self { ifname: value }
    }
}

impl TryFrom<&str> for IfName {
    type Error = IfNameError;

    fn try_from(value: &str) -> std::result::Result<IfName, IfNameError> {
        let value = match value.len() {
            len if len < IF_NAME_MIN => return Err(ErrorKind::TooSmall(value).into()),
            len if len > IF_NAME_MAX => return Err(ErrorKind::TooLarge(value).into()),
            _ => CString::new(value).map_err(|error| ErrorKind::NulError(value, error))?,
        };

        let mut ifname = IfName::new();

        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), ifname.as_mut_ptr(), value.as_bytes().len());
        }

        Ok(ifname)
    }
}
