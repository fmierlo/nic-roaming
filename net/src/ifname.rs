use std::{
    error::Error,
    ffi::{CString, NulError},
    fmt::Display,
    ops::{Deref, DerefMut},
    ptr,
};

const IF_NAME_SIZE: libc::size_t = libc::IFNAMSIZ;

type IfNameType = [libc::c_char; IF_NAME_SIZE];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfNameError {
    error: String,
}

impl IfNameError {
    fn too_small(value: &str) -> IfNameError {
        Self {
            error: format!(
                "value `{}` with length ({}) is too small to fit in IfName length ({})",
                value,
                value.len(),
                IF_NAME_SIZE
            ),
        }
    }

    fn too_large(value: &str) -> IfNameError {
        Self {
            error: format!(
                "value `{}` with length ({}) is too large to fit in IfName length ({})",
                value,
                value.len(),
                IF_NAME_SIZE
            ),
        }
    }

    fn null_error(value: &str, error: NulError) -> IfNameError {
        Self {
            error: format!("error converting value `{}` to CString: {}", value, error),
        }
    }
}

impl Display for IfNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failure converting to IfName, {}", self.error)
    }
}

impl Error for IfNameError {}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct IfName {
    ifname: IfNameType,
}

impl IfName {
    pub fn new() -> Self {
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
        let ifname = crate::str_from_ptr(self.as_ptr()).unwrap_or("error");
        write!(fmt, "{}", ifname)
    }
}

impl TryFrom<&str> for IfName {
    type Error = IfNameError;

    fn try_from(value: &str) -> std::result::Result<IfName, IfNameError> {
        if value.len() == 0 {
            return Err(IfNameError::too_small(value));
        }

        if value.len() > IF_NAME_SIZE {
            return Err(IfNameError::too_large(value));
        }

        let value = CString::new(value).map_err(|error| IfNameError::null_error(value, error))?;

        let mut ifname = IfName::new();

        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), ifname.as_mut_ptr(), value.as_bytes().len());
        }

        Ok(ifname)
    }
}
