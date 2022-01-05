use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Debug)]
pub struct Amount(u64);

impl FromStr for Amount {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Amount(s.parse()?))
    }
}

impl From<u64> for Amount {
    fn from(amount: u64) -> Self {
        Self(amount)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.6}", self.0 as f64 / 1000000.0)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AmountVisitor(PhantomData<fn() -> Amount>);

        impl<'de> Visitor<'de> for AmountVisitor {
            type Value = Amount;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Amount, E>
            where
                E: de::Error,
            {
                Ok(FromStr::from_str(value).unwrap())
            }
        }

        deserializer.deserialize_any(AmountVisitor(PhantomData))
    }
}
