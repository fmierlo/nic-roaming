use std::fmt::{Debug, LowerHex};
use std::ptr::slice_from_raw_parts;

pub(crate) trait AsBytes {
    fn as_bytes(&self) -> &[u8];
    fn as_lower_hex(&self) -> AsLowerHex;
}

impl<T> AsBytes for T {
    fn as_bytes(&self) -> &[u8] {
        let slice = slice_from_raw_parts(self, size_of::<T>());
        return unsafe { std::mem::transmute(slice) };
    }

    fn as_lower_hex(&self) -> AsLowerHex {
        AsLowerHex(self.as_bytes())
    }
}

pub(crate) struct AsLowerHex<'a>(&'a [u8]);

impl<'a> PartialEq for AsLowerHex<'a> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl<'a> LowerHex for AsLowerHex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buf = self
            .0
            .iter()
            .map(|u| format!("{:02x}", u))
            .collect::<String>();
        f.pad_integral(true, "0x", &buf)
    }
}

impl<'a> Debug for AsLowerHex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(&format!("{:#x}", self));
    }
}
