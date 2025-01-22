use std::ops::Deref;
use std::result::Result;
use std::str::FromStr;

use core::fmt::{Debug, Display};

const OCTETS_SIZE: usize = 6;

type OctetsType = [u8; OCTETS_SIZE];

#[derive(Clone, PartialEq, Eq)]
enum Error {
    WrongNumberOfOctets(String, usize),
    InvalidOctet(String, String, String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongNumberOfOctets(value, octets_len) => f
                .debug_struct("LinkLevelAddress::WrongNumberOfOctetsError")
                .field("value", value)
                .field("value_octets", octets_len)
                .field("expected_octets", &OCTETS_SIZE)
                .finish(),
            Self::InvalidOctet(value, octet, error) => f
                .debug_struct("LinkLevelAddress::InvalidOctetError")
                .field("value", value)
                .field("octet", octet)
                .field("error", error)
                .finish(),
        }
    }
}

pub type LLAddr = LinkLevelAddress;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct LinkLevelAddress(OctetsType);

impl Deref for LinkLevelAddress {
    type Target = OctetsType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for LinkLevelAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from(self))
    }
}

impl Display for LinkLevelAddress {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", String::from(self))
    }
}

impl From<&LinkLevelAddress> for String {
    fn from(value: &LinkLevelAddress) -> Self {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            value[0], value[1], value[2], value[3], value[4], value[5]
        )
    }
}

impl From<&OctetsType> for LinkLevelAddress {
    fn from(octets: &OctetsType) -> LinkLevelAddress {
        LinkLevelAddress(*octets)
    }
}

struct OctetsVec(Vec<u8>);

impl Deref for OctetsVec {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&str> for OctetsVec {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let octets = value
            .splitn(OCTETS_SIZE, ':')
            .map(|octet| {
                u8::from_str_radix(octet, 16).map_err(|error| {
                    Error::InvalidOctet(value.to_string(), octet.to_string(), error.to_string())
                })
            })
            .collect::<Result<Vec<u8>, Error>>()?;
        Ok(Self(octets))
    }
}

impl FromStr for LinkLevelAddress {
    type Err = Box<dyn std::error::Error>;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let octets = OctetsVec::try_from(value)?;

        if octets.len() != OCTETS_SIZE {
            return Err(Error::WrongNumberOfOctets(value.to_string(), octets.len()).into());
        }

        let mut lladdr: OctetsType = unsafe { std::mem::zeroed() };
        lladdr.copy_from_slice(&octets);
        Ok(Self::from(&lladdr))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{FromStr, LinkLevelAddress, OctetsType};

    const LLADDR_SIZE: usize = 6;
    const OCTETS: OctetsType = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    #[test]
    fn test_link_level_address_len() {
        let addr = LinkLevelAddress(OCTETS);

        let len = addr.len();

        assert_eq!(len, LLADDR_SIZE);
    }

    #[test]
    fn test_link_level_address_octet_values() {
        let source = OCTETS;

        let addr = LinkLevelAddress(source);

        for i in 0..addr.len() {
            assert_eq!(addr[i], source[i]);
        }
    }

    #[test]
    fn test_link_level_address_copy() {
        let addr = LinkLevelAddress(OCTETS);

        let copy_addr = addr;

        for i in 0..addr.len() {
            assert_eq!(addr[i], copy_addr[i]);
        }
    }

    #[test]
    fn test_link_level_address_clone() {
        let addr = LinkLevelAddress(OCTETS);

        let clone_addr = addr.clone();

        for i in 0..addr.len() {
            assert_eq!(addr[i], clone_addr[i]);
        }
    }

    #[test]
    fn test_link_level_address_partial_eq() {
        let addr = LinkLevelAddress(OCTETS);
        let eq_addr = LinkLevelAddress(OCTETS);

        assert_eq!(addr, eq_addr);
    }

    #[test]
    fn test_link_level_address_partial_ne() {
        let addr = LinkLevelAddress(OCTETS);
        let ne_addr = LinkLevelAddress(unsafe { std::mem::zeroed() });

        assert_ne!(addr, ne_addr);
    }

    #[test]
    fn test_link_level_address_eq_and_hash() {
        let address = LinkLevelAddress(OCTETS);
        let mut map: HashMap<LinkLevelAddress, &str> = HashMap::new();

        map.insert(address, "01:02:03:04:05:06");

        assert_eq!(map.get(&address), Some(&"01:02:03:04:05:06"));
    }

    #[test]
    fn test_link_level_address_as_ptr() {
        let addr = LinkLevelAddress(OCTETS);

        let addr_ptr = addr.as_ptr();

        unsafe {
            for i in 0..addr.len() {
                assert_eq!(addr.get_unchecked(i), &*addr_ptr.add(i));
            }
        }
    }

    #[test]
    fn test_link_level_address_as_ptr_ne_null() {
        let addr = LinkLevelAddress(OCTETS);

        let addr_ptr = addr.as_ptr();

        assert_ne!(addr_ptr, std::ptr::null());
    }

    #[test]
    fn test_link_level_address_display() {
        let addr = LinkLevelAddress(OCTETS);
        let addr_str = format!("{}", addr);
        assert_eq!(addr_str, "01:02:03:04:05:06");
    }

    #[test]
    fn test_link_level_address_debug() {
        let addr = LinkLevelAddress(OCTETS);

        let addr_debug = format!("{:?}", addr);

        assert_eq!(addr_debug, "\"01:02:03:04:05:06\"");
    }

    #[test]
    fn test_link_level_address_from_octets() {
        let source = &OCTETS;
        let expected = LinkLevelAddress(OCTETS);

        let addr = LinkLevelAddress::from(source);

        assert_eq!(addr, expected);
    }

    #[test]
    fn test_link_level_address_from_str() {
        let source = "00:02:03:04:ee:FF";
        let expected = LinkLevelAddress([0x00, 0x02, 0x03, 0x04, 0xEE, 0xff]);

        let addr = LinkLevelAddress::from_str(source).unwrap();

        assert_eq!(addr, expected);
    }

    #[test]
    fn test_link_level_address_from_str_length_too_small() {
        let source = "01:02:03";
        let expected_error = "LinkLevelAddress::WrongNumberOfOctetsError { value: \"01:02:03\", value_octets: 3, expected_octets: 6 }";

        let error = LinkLevelAddress::from_str(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_link_level_address_from_str_length_too_large() {
        let source = "01:02:03:04:05:06:07";
        let expected_error = "LinkLevelAddress::InvalidOctetError { value: \"01:02:03:04:05:06:07\", octet: \"06:07\", error: \"invalid digit found in string\" }";

        let error = LinkLevelAddress::from_str(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_link_level_address_from_str_empty() {
        let source = "";
        let expected_error = "LinkLevelAddress::InvalidOctetError { value: \"\", octet: \"\", error: \"cannot parse integer from empty string\" }";

        let error = LinkLevelAddress::from_str(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_link_level_address_from_str_number_too_large() {
        let source = "01:02:300";
        let expected_error = "LinkLevelAddress::InvalidOctetError { value: \"01:02:300\", octet: \"300\", error: \"number too large to fit in target type\" }";

        let error = LinkLevelAddress::from_str(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }

    #[test]
    fn test_link_level_address_from_str_invalid_digit() {
        let source = "01:02:XX:04:05:06";
        let expected_error = "LinkLevelAddress::InvalidOctetError { value: \"01:02:XX:04:05:06\", octet: \"XX\", error: \"invalid digit found in string\" }";

        let error = LinkLevelAddress::from_str(source).unwrap_err();

        assert_eq!(format!("{}", error), expected_error);
        assert_eq!(format!("{:?}", error), expected_error);
    }
}
