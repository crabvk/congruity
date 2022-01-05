use crate::types::AccountAddress;
use std::fmt;

const ACCOUNT_BALANCE_URL: &str = "https://wallet-proxy.testnet.concordium.com/v0/accBalance";
const DASHBOARD_URL: &str = "https://dashboard.testnet.concordium.com";

pub fn env(env: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get {} env variable", env))
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

    format!(
        r#"<a href="{}/{}">{}</a>"#,
        ACCOUNT_BALANCE_URL, address, addr
    )
}

pub fn txhash_to_hyperlink(hash: &str) -> String {
    format!(
        r#"<a href="{}/lookup/{}">{}</a>"#,
        DASHBOARD_URL,
        hash,
        &hash[..8]
    )
}
