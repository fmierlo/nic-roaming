use super::{ifr, socket::Socket};

#[derive(Default)]
pub struct Nic {
    socket: Box<dyn Socket>,
}

impl Nic {
    pub fn get_mac_address(&self, name: &str) -> String {
        let mut ifr = ifr::new();
        ifr::set_name(&mut ifr, name);

        let _ = match self.socket.open_local_dgram() {
            Ok(s) => s
                .get_lladdr(ifr::to_c_void_ptr(&mut ifr))
                .unwrap_or_default(),
            Err(_) => todo!(),
        };

        let mac_str = ifr::get_mac_address(&ifr);

        mac_str
    }

    pub fn set_mac_address(&self, name: &str, mac_address: &str) -> bool {
        let mut ifr = ifr::new();
        ifr::set_name(&mut ifr, &name);
        ifr::set_mac_address(&mut ifr, &mac_address);

        let _ = match self.socket.open_local_dgram() {
            Ok(s) => s
                .set_lladdr(ifr::to_c_void_ptr(&mut ifr))
                .unwrap_or_default(),
            Err(_) => todo!(),
        };
        true
    }
}

#[cfg(test)]
mod tests {

    use crate::{macos::socket::mock::MockSocket, nic::Nic};

    #[test]
    fn test_get_mac_address() {
        // Given
        let name = "en";
        let expected_mac_address = "00:11:22:33:44:55";

        let socket = MockSocket::default().with_nic(name, expected_mac_address);
        let nic = Nic {
            socket: socket.as_socket(),
        };
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

        let socket = MockSocket::default();
        let nic = Nic {
            socket: socket.as_socket(),
        };
        // When
        let _ = nic.set_mac_address(&name, &mac_address);
        // Then
        assert!(socket.has_nic(&name, &mac_address));
    }
}
