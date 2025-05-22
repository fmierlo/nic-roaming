use std::ffi::c_char;
use std::fmt::{Debug, Display};
use std::ptr::slice_from_raw_parts;

pub(crate) trait AsBytes {
    fn as_bytes(&self) -> &[u8];
    fn as_bytes_ptr(&self) -> *const c_char;
}

impl<T> AsBytes for T {
    fn as_bytes(&self) -> &[u8] {
        let slice = slice_from_raw_parts(self, size_of::<T>());
        unsafe { std::mem::transmute(slice) }
    }

    fn as_bytes_ptr(&self) -> *const c_char {
        let slice = slice_from_raw_parts(self, size_of::<T>());
        unsafe { std::mem::transmute::<*const [T], &[c_char]>(slice) }.as_ptr()
    }
}

pub(crate) trait AsString {
    fn as_string(&self) -> String;
}

impl<T> AsString for T {
    fn as_string(&self) -> String {
        let c_str = unsafe { std::ffi::CStr::from_ptr(self.as_bytes_ptr()) };
        c_str.to_bytes().escape_ascii().to_string()
    }
}

pub(crate) trait AsHexColon {
    fn as_hex_colon(&self) -> HexColon;
}

impl<T> AsHexColon for T {
    fn as_hex_colon(&self) -> HexColon {
        HexColon(self.as_bytes())
    }
}

pub(crate) struct HexColon<'a>(&'a [u8]);

impl<'a> HexColon<'a> {
    fn to_string(&self) -> String {
        self.0
            .iter()
            .map(|u| format!("{:02x}", u))
            .collect::<Vec<String>>()
            .join(":")
    }
}

impl<'a> From<HexColon<'a>> for String {
    fn from(value: HexColon<'a>) -> Self {
        value.to_string()
    }
}

impl<'a> PartialEq for HexColon<'a> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<'a> Display for HexColon<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<'a> Debug for HexColon<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}
