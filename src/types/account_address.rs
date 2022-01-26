use base58check::{FromBase58Check, ToBase58Check};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;
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

pub fn account_address_hex_or_struct<'de, D>(deserializer: D) -> Result<AccountAddress, D::Error>
where
    D: Deserializer<'de>,
{
    struct AccountAddressVisitor(PhantomData<fn() -> AccountAddress>);

    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Field {
        Type,
        Address,
    }

    impl<'de> Visitor<'de> for AccountAddressVisitor {
        type Value = AccountAddress;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("hex string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<AccountAddress, E>
        where
            E: de::Error,
        {
            let address = hex::decode(&value[2..]).unwrap().to_base58check(1);
            Ok(AccountAddress(address))
        }

        fn visit_map<M>(self, mut map: M) -> Result<AccountAddress, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut r#type = None;
            let mut address = None;
            while let Some(key) = map.next_key()? {
                match key {
                    Field::Type => {
                        r#type = Some(map.next_value()?);
                    }
                    Field::Address => {
                        address = Some(map.next_value()?);
                    }
                }
            }
            let r#type: String = r#type.ok_or_else(|| de::Error::missing_field("type"))?;
            let address = address.ok_or_else(|| de::Error::missing_field("address"))?;

            if r#type != "AddressAccount" {
                Err(de::Error::custom(
                    "Expected field \"type\" to be \"AddressAccount\"",
                ))
            } else {
                Ok(AccountAddress(address))
            }
        }
    }

    deserializer.deserialize_any(AccountAddressVisitor(PhantomData))
}
