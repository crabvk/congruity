use base58check::FromBase58Check;
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

pub struct ParseAccountAddressError;

impl fmt::Display for ParseAccountAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid account address")
    }
}

#[derive(Deserialize, Debug)]
pub struct AccountAddress(String);

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for AccountAddress {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl FromStr for AccountAddress {
    type Err = ParseAccountAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok((1, bytes)) = s.from_base58check() {
            if bytes.len() == 32 {
                Ok(AccountAddress(s.to_string()))
            } else {
                Err(ParseAccountAddressError)
            }
        } else {
            Err(ParseAccountAddressError)
        }
    }
}

impl AccountAddress {
    pub fn new(address: String) -> Self {
        Self(address)
    }

    pub fn address(&self) -> &str {
        &self.0
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let (_, bytes) = self.0.from_base58check().unwrap();
        bytes
    }
}
