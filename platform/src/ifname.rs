#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct IfName {
    name: [libc::c_char; libc::IFNAMSIZ],
}

impl IfName {
    pub fn len(&self) -> usize {
        self.name.len()
    }

    pub fn as_ptr(&self) -> *const i8 {
        self.name.as_ptr()
    }
}
