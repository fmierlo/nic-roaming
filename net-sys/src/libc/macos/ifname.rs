use core::fmt::{Debug, Display};
use std::{ffi::CString, ops::Deref, ptr};

const IF_NAME_SIZE: libc::size_t = libc::IFNAMSIZ;
const IF_NAME_MIN: libc::size_t = 3;
const IF_NAME_MAX: libc::size_t = IF_NAME_SIZE - 1;

type IfNameType = [libc::c_char; IF_NAME_SIZE];

#[derive(Clone, PartialEq, Eq)]
enum Error {
    TooSmall(String),
    TooLarge(String),
    InvalidCString(String, String),
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
            Self::InvalidCString(value, error) => f
                .debug_struct("IfName::InvalidCStringError")
                .field("value", value)
                .field("error", error)
                .finish(),
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct IfName(IfNameType);

impl Deref for IfName {
    type Target = IfNameType;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl From<&IfNameType> for IfName {
    fn from(value: &IfNameType) -> Self {
        Self(*value)
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
            _ => CString::new(value.clone())
                .map_err(|error| Error::InvalidCString(value, error.to_string()))?,
        };

        let mut ifname: IfNameType = unsafe { std::mem::zeroed() };
        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), ifname.as_mut_ptr(), value.as_bytes().len());
        }
        Ok(Self::from(&ifname))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{IfName, IfNameType};

    const IF_NAME_SIZE: usize = 16;
    const IF_NAME: IfNameType = [
        // '0'..'9' and 'A'..'F'
        0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x41, 0x42, 0x43, 0x44, 0x45,
        0x00,
    ];

    #[test]
    fn test_ifname_len() {
        let ifname = IfName(IF_NAME);

        let len = ifname.len();

        assert_eq!(len, IF_NAME_SIZE);
    }

    #[test]
    fn test_ifname_char_values() {
        let source = IF_NAME;

        let ifname = IfName(source);

        for i in 0..ifname.len() {
            assert_eq!(ifname[i], source[i]);
        }
    }

    #[test]
    fn test_ifname_copy() {
        let ifname = IfName(IF_NAME);

        let copy_ifname = ifname;

        for i in 0..ifname.len() {
            assert_eq!(ifname[i], copy_ifname[i]);
        }
    }

    #[test]
    fn test_ifname_clone() {
        let ifname = IfName(IF_NAME);

        let clone_ifname = ifname.clone();

        for i in 0..ifname.len() {
            assert_eq!(ifname[i], clone_ifname[i]);
        }
    }

    #[test]
    fn test_ifname_partial_eq() {
        let ifname = IfName(IF_NAME);
        let eq_ifname = IfName(IF_NAME);

        assert_eq!(ifname, eq_ifname);
    }

    #[test]
    fn test_ifname_partial_ne() {
        let ifname = IfName(IF_NAME);
        let ne_ifname = IfName(unsafe { std::mem::zeroed() });

        assert_ne!(ifname, ne_ifname);
    }

    #[test]
    fn test_ifname_eq_and_hash() {
        let ifname = IfName(IF_NAME);
        let mut map: HashMap<IfName, &str> = HashMap::new();

        map.insert(ifname, "interface");

        assert_eq!(map.get(&ifname), Some(&"interface"));
    }

    #[test]
    fn test_ifname_as_ptr() {
        let ifname = IfName(IF_NAME);

        let ifname_ptr = ifname.as_ptr();

        unsafe {
            for i in 0..ifname.len() {
                assert_eq!(ifname.get_unchecked(i), &*ifname_ptr.add(i));
            }
        }
    }

    #[test]
    fn test_ifname_as_ptr_ne_null() {
        let ifname = IfName(IF_NAME);

        let ifname_ptr = ifname.as_ptr();

        assert_ne!(ifname_ptr, std::ptr::null());
    }

    #[test]
    fn test_ifname_display() {
        let ifname = IfName(IF_NAME);

        let ifname_str = format!("{}", ifname);

        assert_eq!(ifname_str, "0123456789ABCDE");
    }

    #[test]
    fn test_ifname_debug() {
        let ifname = IfName(IF_NAME);

        let ifname_debug = format!("{:?}", ifname);

        assert_eq!(ifname_debug, "\"0123456789ABCDE\"");
    }

    #[test]
    fn test_ifname_from_octets() {
        let source = &IF_NAME;
        let expected = IfName(IF_NAME);

        let ifname = IfName::from(source);

        assert_eq!(ifname, expected);
    }

    #[test]
    fn test_ifname_from_str() {
        let source = "0123456789ABCDE";
        let expected = IfName(IF_NAME);

        let ifname = IfName::try_from(source).unwrap();

        assert_eq!(ifname, expected);
    }

    #[test]
    fn test_ifname_from_str_length_too_small() {
        let source = "en";
        let expected_error = "IfName::TooSmallError { value: \"en\", len: 2, min: 3 }";

        let error = IfName::try_from(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_ifname_from_str_length_too_large() {
        let source = "0123456789ABCDEF";
        let expected_error =
            "IfName::TooLargeError { value: \"0123456789ABCDEF\", len: 16, max: 15 }";

        let error = IfName::try_from(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_ifname_from_str_empty() {
        let source = "";
        let expected_error = "IfName::TooSmallError { value: \"\", len: 0, min: 3 }";

        let error = IfName::try_from(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_ifname_from_str_nul_error() {
        let source = "0123456\089ABCDE";
        let expected_error = "IfName::InvalidCStringError { value: \"0123456\\089ABCDE\", error: \"nul byte found in provided data at position: 7\" }";

        let error = IfName::try_from(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }
}
