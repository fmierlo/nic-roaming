use libc::c_ulong;

use super::ioc;

// https://github.com/apple/darwin-xnu/blob/xnu-7195.121.3/bsd/sys/sockio.h

pub(crate) const IFREQ_SIZE: c_ulong = 32;

// Get link level addr
// SIOCGIFLLADDR = (0x80000000 |0x40000000) | 32 << 16 | (105 << 8) | 158 = 0xc020699e
pub(crate) const SIOCGIFLLADDR: c_ulong = ioc::iorw(ioc::I, 158, IFREQ_SIZE);

// Set link level addr
// SIOCSIFLLADDR = 0x80000000 | 32 << 16 | (105 << 8) | 60 = 0x8020693c
pub(crate) const SIOCSIFLLADDR: c_ulong = ioc::iow(ioc::I, 60, IFREQ_SIZE);

#[cfg(test)]
mod tests {
    use libc::c_ulong;

    use crate::Result;

    use super::{IFREQ_SIZE, SIOCGIFLLADDR, SIOCSIFLLADDR};

    #[test]
    fn test_ifreq_size() -> Result<()> {
        let expected_size: c_ulong = std::mem::size_of::<libc::ifreq>().try_into()?;

        assert_eq!(IFREQ_SIZE, expected_size);

        Ok(())
    }

    #[test]
    fn test_get_link_level_addr() {
        assert_eq!(SIOCGIFLLADDR, 0xc020699e)
    }

    #[test]
    fn test_set_link_level_addr() {
        assert_eq!(SIOCSIFLLADDR, 0x8020693c)
    }
}
