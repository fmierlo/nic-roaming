use core::fmt;
use std::{error::Error, ops::Deref, result::Result, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLinkLevelAddressError {
    pub source: String,
    pub error: String,
}

impl fmt::Display for ParseLinkLevelAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse `{}` as LinkLevelAddr, {}",
            self.source, self.error
        )
    }
}

impl Error for ParseLinkLevelAddressError {}

pub type LLAddr = LinkLevelAddress;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct LinkLevelAddress {
    octets: [u8; 6],
}

impl Deref for LinkLevelAddress {
    type Target = [u8; 6];

    fn deref(&self) -> &Self::Target {
        &self.octets
    }
}

impl fmt::Display for LinkLevelAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self[0], self[1], self[2], self[3], self[4], self[5]
        )
    }
}

impl fmt::Debug for LinkLevelAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl From<&[u8; 6]> for LinkLevelAddress {
    fn from(octets: &[u8; 6]) -> LinkLevelAddress {
        LinkLevelAddress { octets: *octets }
    }
}

fn from_str_radix_16(source: &str, token: &str) -> Result<u8, ParseLinkLevelAddressError> {
    match u8::from_str_radix(token, 16) {
        Ok(value) => Ok(value),
        Err(error) => Err(ParseLinkLevelAddressError {
            source: source.to_string(),
            error: format!("error in token `{}`: {}", token, error),
        }),
    }
}

impl FromStr for LinkLevelAddress {
    type Err = ParseLinkLevelAddressError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let mut octets = [0u8; 6];

        let tokens = source
            .splitn(octets.len(), ':')
            .map(|token| from_str_radix_16(source, token))
            .collect::<Result<Vec<u8>, Self::Err>>()?;

        if tokens.len() != octets.len() {
            return Err(ParseLinkLevelAddressError {
                source: source.to_string(),
                error: format!(
                    "source tokens length ({}) does not match LinkLevelAddress length ({})",
                    tokens.len(),
                    octets.len()
                ),
            });
        }

        octets.copy_from_slice(&tokens);

        Ok(Self::from(&octets))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const OCTETS: [u8; 6] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];

    #[test]
    fn test_link_level_address_octet_values() {
        let addr = LinkLevelAddress { octets: OCTETS };
        assert_eq!(addr.octets[0], 0x01);
        assert_eq!(addr.octets[1], 0x02);
        assert_eq!(addr.octets[2], 0x03);
        assert_eq!(addr.octets[3], 0x04);
        assert_eq!(addr.octets[4], 0x05);
        assert_eq!(addr.octets[5], 0x06);
    }

    #[test]
    fn test_link_level_address_copy() {
        let addr = LinkLevelAddress { octets: OCTETS };
        let copy_addr = addr;
        assert_eq!(addr.octets[0], copy_addr.octets[0]);
        assert_eq!(addr.octets[1], copy_addr.octets[1]);
        assert_eq!(addr.octets[2], copy_addr.octets[2]);
        assert_eq!(addr.octets[3], copy_addr.octets[3]);
        assert_eq!(addr.octets[4], copy_addr.octets[4]);
        assert_eq!(addr.octets[5], copy_addr.octets[5]);
    }

    #[test]
    fn test_link_level_address_clone() {
        let addr = LinkLevelAddress { octets: OCTETS };
        let clone_addr = addr.clone();
        assert_eq!(addr.octets[0], clone_addr.octets[0]);
        assert_eq!(addr.octets[1], clone_addr.octets[1]);
        assert_eq!(addr.octets[2], clone_addr.octets[2]);
        assert_eq!(addr.octets[3], clone_addr.octets[3]);
        assert_eq!(addr.octets[4], clone_addr.octets[4]);
        assert_eq!(addr.octets[5], clone_addr.octets[5]);
    }

    #[test]
    fn test_link_level_address_partial_eq() {
        let addr = LinkLevelAddress { octets: OCTETS };
        let eq_addr = LinkLevelAddress { octets: OCTETS };
        assert_eq!(addr, eq_addr);
    }

    #[test]
    fn test_link_level_address_partial_ne() {
        let addr = LinkLevelAddress { octets: OCTETS };
        let ne_addr = LinkLevelAddress {
            octets: [0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
        };
        assert_ne!(addr, ne_addr);
    }

    #[test]
    fn test_link_level_address_eq_and_hash() {
        let address = LinkLevelAddress { octets: OCTETS };
        let mut map: HashMap<LinkLevelAddress, &str> = HashMap::new();
        map.insert(address, "01:02:03:04:05:06");

        assert_eq!(map.get(&address), Some(&"01:02:03:04:05:06"));
    }

    #[test]
    fn test_link_level_address_partial_update() {
        let mut addr = LinkLevelAddress { octets: OCTETS };
        addr.octets[2] = 0x30;
        assert_eq!(addr.octets[0], 0x01);
        assert_eq!(addr.octets[1], 0x02);
        assert_eq!(addr.octets[2], 0x30);
        assert_eq!(addr.octets[3], 0x04);
        assert_eq!(addr.octets[4], 0x05);
        assert_eq!(addr.octets[5], 0x06);
    }

    #[test]
    fn test_link_level_address_fill() {
        let mut addr = LinkLevelAddress { octets: OCTETS };
        addr.octets.fill(0x10);
        assert_eq!(addr.octets[0], 0x10);
        assert_eq!(addr.octets[1], 0x10);
        assert_eq!(addr.octets[2], 0x10);
        assert_eq!(addr.octets[3], 0x10);
        assert_eq!(addr.octets[4], 0x10);
        assert_eq!(addr.octets[5], 0x10);
    }

    #[test]
    fn test_link_level_address_len() {
        let addr = LinkLevelAddress { octets: OCTETS };
        assert_eq!(addr.len(), 6);
    }

    #[test]
    fn test_link_level_address_as_ptr() {
        let addr = LinkLevelAddress { octets: OCTETS };

        let addr_ptr = addr.as_ptr();

        unsafe {
            for i in 0..addr.octets.len() {
                assert_eq!(addr.octets.get_unchecked(i), &*addr_ptr.add(i));
            }
        }
    }

    #[test]
    fn test_link_level_address_as_ptr_ne_null() {
        let addr = LinkLevelAddress { octets: OCTETS };

        assert_ne!(addr.as_ptr(), std::ptr::null());
    }

    #[test]
    fn test_link_level_address_display() {
        let addr = LinkLevelAddress { octets: OCTETS };
        assert_eq!(format!("{}", addr), "01:02:03:04:05:06");
    }

    #[test]
    fn test_link_level_address_debug() {
        let addr = LinkLevelAddress { octets: OCTETS };
        assert_eq!(format!("{:?}", addr), "01:02:03:04:05:06");
    }

    #[test]
    fn test_link_level_address_from_octets() {
        let source = &OCTETS;
        let expected = LinkLevelAddress { octets: OCTETS };

        assert_eq!(LinkLevelAddress::from(source), expected);
    }

    #[test]
    fn test_link_level_address_from_str() {
        let source = "00:02:03:04:ee:FF";
        let expected = LinkLevelAddress {
            octets: [0x00, 0x02, 0x03, 0x04, 0xEE, 0xff],
        };

        assert_eq!(LinkLevelAddress::from_str(source).unwrap(), expected);
    }

    #[test]
    fn test_link_level_address_from_str_format_error() {
        let error = ParseLinkLevelAddressError {
            source: "source".to_string(),
            error: "error".to_string(),
        };

        assert_eq!(
            format!("{}", error),
            "Failed to parse `source` as LinkLevelAddr, error"
        );
    }

    #[test]
    fn test_link_level_address_from_str_length_too_small() {
        let source = "01:02:03";
        let error = ParseLinkLevelAddressError {
            source: source.to_string(),
            error: "source tokens length (3) does not match LinkLevelAddress length (6)"
                .to_string(),
        };

        assert_eq!(LinkLevelAddress::from_str(source), Err(error));
    }

    #[test]
    fn test_link_level_address_from_str_length_too_large() {
        let source = "01:02:03:04:05:06:07";
        let error = ParseLinkLevelAddressError {
            source: source.to_string(),
            error: "error in token `06:07`: invalid digit found in string".to_string(),
        };

        assert_eq!(LinkLevelAddress::from_str(source), Err(error));
    }

    #[test]
    fn test_link_level_address_from_str_empty() {
        let source = "";
        let error = ParseLinkLevelAddressError {
            source: source.to_string(),
            error: "error in token ``: cannot parse integer from empty string".to_string(),
        };

        assert_eq!(LinkLevelAddress::from_str(source), Err(error));
    }

    #[test]
    fn test_link_level_address_from_str_number_too_large() {
        let source = "01:02:300";
        let error = ParseLinkLevelAddressError {
            source: source.to_string(),
            error: "error in token `300`: number too large to fit in target type".to_string(),
        };

        assert_eq!(LinkLevelAddress::from_str(source), Err(error));
    }

    #[test]
    fn test_link_level_address_from_str_invalid_digit() {
        let source = "01:02:XX:04:05:06";
        let error = ParseLinkLevelAddressError {
            source: source.to_string(),
            error: "error in token `XX`: invalid digit found in string".to_string(),
        };

        assert_eq!(LinkLevelAddress::from_str(source), Err(error));
    }
}
