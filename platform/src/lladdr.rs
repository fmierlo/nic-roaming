use core::fmt;
use std::{error::Error, num::ParseIntError, result::Result, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLinkLevelAddressError {
    pub source: String,
    pub token: String,
    pub error: ParseIntError,
}

impl fmt::Display for ParseLinkLevelAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse `{}` as LinkLevelAddr, token `{}` error: {}",
            self.source, self.token, self.error
        )
    }
}

impl Error for ParseLinkLevelAddressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

pub type LLAddr = LinkLevelAddress;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LinkLevelAddress {
    octets: [u8; 6],
}

impl From<[u8; 6]> for LinkLevelAddress {
    fn from(octets: [u8; 6]) -> LinkLevelAddress {
        LinkLevelAddress { octets }
    }
}

fn from_str_radix_16(source: &str, token: &str) -> Result<u8, ParseLinkLevelAddressError> {
    match u8::from_str_radix(token, 16) {
        Ok(value) => Ok(value),
        Err(error) => Err(ParseLinkLevelAddressError {
            source: source.to_string(),
            token: token.to_string(),
            error,
        }),
    }
}

impl FromStr for LinkLevelAddress {
    type Err = ParseLinkLevelAddressError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let source = source
            .splitn(6, ':')
            .map(|token| from_str_radix_16(source, token))
            .collect::<Result<Vec<u8>, Self::Err>>()?;

        let mut octets = [0u8; 6];
        octets.copy_from_slice(&source);

        Ok(Self::from(octets))
    }
}

impl fmt::Display for LinkLevelAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octets = self.octets;

        write!(
            fmt,
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            octets[0], octets[1], octets[2], octets[3], octets[4], octets[5]
        )
    }
}

impl fmt::Debug for LinkLevelAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}
