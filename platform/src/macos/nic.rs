use super::{
    ifr::{Ifr, IfrGet, IfrSet},
    socket::DynSocket,
};
#[derive(Debug, Default)]
pub struct Nic {
    socket: DynSocket,
}

impl Nic {
    pub fn get_mac_address(&self, name: &str) -> String {
        let mut ifr: Ifr = IfrGet { name }.into();

        let _ = match self.socket.open_local_dgram() {
            Ok(socket) => socket.get_lladdr(ifr.as_mut_ptr()).unwrap_or_default(),
            Err(_) => todo!(),
        };

        ifr.mac_address()
    }

    pub fn set_mac_address(&self, name: &str, mac_address: &str) -> bool {
        let mut ifr: Ifr = IfrSet { name, mac_address }.into();

        let _ = match self.socket.open_local_dgram() {
            Ok(socket) => socket.set_lladdr(ifr.as_mut_ptr()).unwrap_or_default(),
            Err(_) => todo!(),
        };
        true
    }
}

#[cfg(test)]
mod tests {

    use crate::{macos::socket::mock::MockSocket, nic::Nic};

    use super::DynSocket;

    impl Nic {
        fn new(socket: &MockSocket) -> Nic {
            Nic {
                socket: DynSocket(Box::new(socket.clone())),
            }
        }
    }

    #[test]
    fn test_get_mac_address() {
        // Given
        let name = "en";
        let expected_mac_address = "00:11:22:33:44:55";

        let socket = MockSocket::default().with_nic(name, expected_mac_address);
        // When
        let mac_address = Nic::new(&socket).get_mac_address(&name);
        // Then
        assert_eq!(mac_address, expected_mac_address)
    }

    #[test]
    fn test_set_mac_address() {
        // Given
        let name = "en";
        let mac_address = "00:11:22:33:44:55";

        let socket = MockSocket::default();
        // When
        let _ = Nic::new(&socket).set_mac_address(&name, &mac_address);
        // Then
        assert!(socket.has_nic(&name, &mac_address));
    }
}
