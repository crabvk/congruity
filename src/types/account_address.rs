use base58check::FromBase58Check;
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
    pub fn to_bytes(&self) -> Vec<u8> {
        let (_, bytes) = self.0.from_base58check().unwrap();
        bytes
    }
}

pub fn account_address_from_struct<'de, D>(deserializer: D) -> Result<AccountAddress, D::Error>
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
            formatter.write_str("map")
        }

        fn visit_map<V>(self, mut map: V) -> Result<AccountAddress, V::Error>
        where
            V: MapAccess<'de>,
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
