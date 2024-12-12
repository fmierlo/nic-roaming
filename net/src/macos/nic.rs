use crate::{LinkLevelAddress, Result};

use super::{
    ifreq::{self},
    socket::BoxSocket,
};

#[derive(Debug, Default)]
pub struct Nic {
    socket: BoxSocket,
}

impl Nic {
    pub fn get_lladd(&self, name: &str) -> Result<LinkLevelAddress> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &name)?;

        self.socket
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        ifreq::get_lladdr(&ifreq)
    }

    pub fn set_lladd(&self, name: &str, lladdr: &LinkLevelAddress) -> Result<()> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, &name)?;
        ifreq::set_lladdr(&mut ifreq, lladdr)?;

        self.socket
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))
    }
}

#[cfg(test)]
mod tests {

    use crate::{macos::socket::mock::MockSocket, LLAddr, Nic, Result};

    use super::BoxSocket;

    impl Nic {
        fn new(socket: &MockSocket) -> Nic {
            Nic {
                socket: BoxSocket(Box::new(socket.clone())),
            }
        }
    }

    #[test]
    fn test_get_lladd() -> Result<()> {
        // Given
        let name = "en";
        let expected_lladd: LLAddr = "00:11:22:33:44:55".parse()?;

        let socket = MockSocket::default().with_nic(name, &expected_lladd);
        // When
        let lladd = Nic::new(&socket).get_lladd(&name)?;
        // Then
        assert_eq!(lladd, expected_lladd);

        Ok(())
    }

    #[test]
    fn test_set_lladd() -> Result<()> {
        // Given
        let name = "en";
        let lladd: LLAddr = "00:11:22:33:44:55".parse()?;

        let socket = MockSocket::default();
        // When
        Nic::new(&socket).set_lladd(&name, &lladd)?;
        // Then
        assert!(socket.has_nic(&name, &lladd));
        Ok(())
    }
}
