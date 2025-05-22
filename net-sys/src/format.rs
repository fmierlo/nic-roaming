use std::fmt::Debug;
use std::ptr::slice_from_raw_parts;

pub(crate) trait AsBytes {
    fn as_bytes(&self) -> &[u8];
    fn as_hex_colon(&self) -> AsHexColon;
}

impl<T> AsBytes for T {
    fn as_bytes(&self) -> &[u8] {
        let slice = slice_from_raw_parts(self, size_of::<T>());
        return unsafe { std::mem::transmute(slice) };
    }

    fn as_hex_colon(&self) -> AsHexColon {
        AsHexColon(self.as_bytes())
    }
}

pub(crate) struct AsHexColon<'a>(&'a [u8]);

impl<'a> From<AsHexColon<'a>> for String {
    fn from(value: AsHexColon<'a>) -> Self {
        value.to_string()
    }
}

impl<'a> PartialEq for AsHexColon<'a> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<'a> ToString for AsHexColon<'a> {
    fn to_string(&self) -> String {
        self
            .0
            .iter()
            .map(|u| format!("{:02x}", u))
            .collect::<Vec<String>>()
            .join(":")
    }
}

impl<'a> Debug for AsHexColon<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}
