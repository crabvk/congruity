use crate::types::AccountAddress;
use std::fmt;

const MAINNET_DASHBOARD_URL: &str = "http://dashboard.mainnet.concordium.software";
const MAINNET_API_URL: &str = "https://wallet-proxy.mainnet.concordium.software/v0";

const TESTNET_DASHBOARD_URL: &str = "https://dashboard.testnet.concordium.com";
const TESTNET_API_URL: &str = "https://wallet-proxy.testnet.concordium.com/v0";

pub fn env(env: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get {} env variable", env))
}

pub fn is_mainnet() -> bool {
    let value = std::env::var("CONGRUITY_MAINNET").unwrap_or_default();
    let value = &value[..];

    if ["true", "1"].contains(&value) {
        true
    } else {
        false
    }
}

pub enum Emoji {
    Person,
    #[allow(dead_code)]
    Robot,
}

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let emoji = match self {
            Emoji::Person => '👤',
            Emoji::Robot => '🤖',
        };
        write!(f, "{}", emoji)
    }
}

pub fn address_to_hyperlink(address: &AccountAddress, emoji: Option<Emoji>) -> String {
    let addr_str = address.to_string();
    let addr = if let Some(emoji) = emoji {
        format!("{}{}", emoji, &addr_str[..8])
    } else {
        addr_str[..8].to_string()
    };

    let url = if is_mainnet() {
        MAINNET_API_URL
    } else {
        TESTNET_API_URL
    };

    format!(r#"<a href="{}/accBalance/{}">{}</a>"#, url, address, addr)
}

pub fn txhash_to_hyperlink(hash: &str) -> String {
    let url = if is_mainnet() {
        MAINNET_DASHBOARD_URL
    } else {
        TESTNET_DASHBOARD_URL
    };

    format!(r#"<a href="{}/lookup/{}">{}</a>"#, url, hash, &hash[..8])
}
