use crate::{IfName, LinkLevelAddress, Result};

use super::{
    ifreq::{self},
    socket::BoxSocket,
};

#[derive(Debug, Default)]
pub struct Nic {
    socket: BoxSocket,
}

impl Nic {
    pub fn get_lladd(&self, ifname: &IfName) -> Result<LinkLevelAddress> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, ifname);

        self.socket
            .open_local_dgram()?
            .get_lladdr(ifreq::as_mut_ptr(&mut ifreq))?;

        Ok(ifreq::get_lladdr(&ifreq))
    }

    pub fn set_lladd(&self, ifname: &IfName, lladdr: &LinkLevelAddress) -> Result<()> {
        let mut ifreq = ifreq::new();
        ifreq::set_name(&mut ifreq, ifname);
        ifreq::set_lladdr(&mut ifreq, lladdr);

        self.socket
            .open_local_dgram()?
            .set_lladdr(ifreq::as_mut_ptr(&mut ifreq))
    }
}

#[cfg(test)]
mod tests {

    use crate::{macos::socket::mock::MockSocket, IfName, LLAddr, Nic, Result};

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
        let ifname: IfName = "en".try_into()?;
        let expected_lladdr: LLAddr = "00:11:22:33:44:55".parse()?;

        let socket = MockSocket::default().with_nic(ifname, expected_lladdr);
        // When
        let lladdr = Nic::new(&socket).get_lladd(&ifname)?;
        // Then
        assert_eq!(lladdr, expected_lladdr);

        Ok(())
    }

    #[test]
    fn test_set_lladd() -> Result<()> {
        // Given
        let ifname: IfName = "en".try_into()?;
        let lladdr: LLAddr = "00:11:22:33:44:55".parse()?;

        let socket = MockSocket::default();
        // When
        Nic::new(&socket).set_lladd(&ifname, &lladdr)?;
        // Then
        assert!(socket.has_nic(&ifname, &lladdr));
        Ok(())
    }
}
