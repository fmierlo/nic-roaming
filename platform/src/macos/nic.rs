use std::io::Error as IOError;
use std::rc::Rc;

use super::{ifr, sys};

pub fn new() -> Nic<sys::LibcSys> {
    Nic::new(&sys::new())
}

pub struct Nic<T: sys::Sys> {
    sys: Rc<T>,
}

impl<T> Nic<T>
where
    T: sys::Sys,
{
    fn new(sys: &Rc<T>) -> Nic<T> {
        Nic {
            sys: Rc::clone(sys),
        }
    }

    pub fn get_mac_address(&self, name: &str) -> String {
        println!("get_mac_address({})", name);

        let mut ifr = ifr::new();
        ifr::set_name(&mut ifr, name);

        let s = self.sys.socket(libc::AF_LOCAL, libc::SOCK_DGRAM, 0);
        error("socket", s);

        let ret = self
            .sys
            .ioctl(s, sys::SIOCGIFLLADDR, ifr::to_c_void_ptr(&mut ifr));
        error("ioctl", ret);

        let ret = self.sys.close(s);
        error("close", ret);

        let mac_str = ifr::get_mac_address(&ifr);

        println!("get_mac_address({mac_str})");

        mac_str
    }

    pub fn set_mac_address(&self, name: &str, mac_address: &str) -> bool {
        println!("set_mac_address({}, {mac_address})", name);

        let mut ifr = ifr::new();
        ifr::set_name(&mut ifr, &name);
        ifr::set_mac_address(&mut ifr, &mac_address);

        let s = self.sys.socket(libc::AF_LOCAL, libc::SOCK_DGRAM, 0);
        error("socket", s);

        let ret = self
            .sys
            .ioctl(s, sys::SIOCSIFLLADDR, ifr::to_c_void_ptr(&mut ifr));
        error("ioctl", ret);

        let ret = self.sys.close(s);
        error("close", ret);

        true
    }
}

fn error(prefix: &str, s: libc::c_int) {
    if s < 0 {
        let err = IOError::last_os_error();
        println!("{prefix}: {s} {err}");
    }
}

#[cfg(test)]
mod tests {

    use crate::{macos::sys, nic::Nic};

    use sys::tests::MockSys;

    #[test]
    fn test_get_mac_address() {
        // Given
        let name = "en";
        let expected_mac_address = "00:11:22:33:44:55";

        let mocked_sys = MockSys::new().with_nic(name, expected_mac_address);
        let nic = Nic::new(&mocked_sys);
        // When
        let mac_address = nic.get_mac_address(&name);
        // Then
        assert_eq!(mac_address, expected_mac_address)
    }

    #[test]
    fn test_set_mac_address() {
        // Given
        let name = "en";
        let mac_address = "00:11:22:33:44:55";

        let mocked_sys = MockSys::new();
        let nic = Nic::new(&mocked_sys);
        // When
        let _ = nic.set_mac_address(&name, &mac_address);
        // Then
        assert!(mocked_sys.has_nic(&name, &mac_address));
    }
}
