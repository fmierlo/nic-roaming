use core::fmt;
use std::{error::Error, result::Result, str::FromStr};

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

impl Error for ParseLinkLevelAddressError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub type LLAddr = LinkLevelAddress;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LinkLevelAddress {
    octets: [u8; 6],
}

impl LinkLevelAddress {
    pub fn as_ptr(&self) -> *const u8 {
        self.octets.as_ptr()
    }

    pub fn octets(&self) -> &[u8; 6] {
        &self.octets
    }
}

impl From<&[u8; 6]> for LinkLevelAddress {
    fn from(octets: &[u8; 6]) -> LinkLevelAddress {
        LinkLevelAddress {
            octets: octets.clone(),
        }
    }
}

fn from_str_radix_16(source: &str, token: &str) -> Result<u8, ParseLinkLevelAddressError> {
    match u8::from_str_radix(token, 16) {
        Ok(value) => Ok(value),
        Err(error) => Err(ParseLinkLevelAddressError {
            source: source.to_string(),
            error: format!("token `{}` error: {}", token, error),
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

impl fmt::Display for LinkLevelAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octets = self.octets;

        write!(
            fmt,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            octets[0], octets[1], octets[2], octets[3], octets[4], octets[5]
        )
    }
}

impl fmt::Debug for LinkLevelAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}
