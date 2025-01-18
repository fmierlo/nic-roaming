use super::ifname::IfName;
use super::ifreq::{self};
use super::socket::BoxSocket;
use crate::{LinkLevelAddress, Result};

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
    use super::{BoxSocket, IfName, Nic};
    use crate::sys::os::ifreq::mock::{ifreq_get_lladdr, ifreq_get_name, ifreq_set_lladdr};
    use crate::sys::os::socket::mock::{self, mock, ErrNo, MockSocket};
    use crate::{LinkLevelAddress, Result};
    use mockdown::ThreadLocal;
    use std::sync::LazyLock;

    impl Nic {
        fn new(socket: &MockSocket) -> Nic {
            Nic {
                socket: BoxSocket(Box::new(socket.clone())),
            }
        }
    }

    static IFNAME: LazyLock<IfName> = LazyLock::new(|| "enx".try_into().unwrap());
    static LLADDR: LazyLock<LinkLevelAddress> =
        LazyLock::new(|| "00:11:22:33:44:55".parse().unwrap());

    #[test]
    fn test_nic_default() {
        let expected_default = "Nic { socket: BoxSocket(LibcSocket) }";

        let nic = super::Nic::default();

        assert_eq!(format!("{:?}", nic), expected_default);
    }

    #[test]
    fn test_get_lladd() {
        mock()
            .expect(|mock::OpenLocalDgram()| {
                assert!(true);
                ErrNo::None
            })
            .expect(|mock::GetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                ifreq_set_lladdr(ifreq, *LLADDR);
                Result::Ok(())
            });

        let socket = MockSocket::default();
        let lladdr = Nic::new(&socket).get_lladd(&IFNAME).unwrap();

        assert_eq!(lladdr, *LLADDR);
    }

    #[test]
    fn test_set_lladd() {
        mock()
            .expect(|mock::OpenLocalDgram()| {
                assert!(true);
                ErrNo::None
            })
            .expect(|mock::SetLLAddr(ifreq)| {
                assert_eq!(ifreq_get_name(ifreq), *IFNAME);
                assert_eq!(ifreq_get_lladdr(ifreq), *LLADDR);
                Result::Ok(())
            });

        let socket = MockSocket::default();
        Nic::new(&socket).set_lladd(&IFNAME, &LLADDR).unwrap();
    }
}
