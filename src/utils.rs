use crate::types::{AccountAddress, Address};
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

enum Emoji {
    Account,
    Contract,
}

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let emoji = match self {
            Emoji::Account => '👤',
            Emoji::Contract => '📝',
        };
        write!(f, "{}", emoji)
    }
}

pub fn format_address(address: &Address) -> String {
    match address {
        Address::Account(account) => format_account_address(account, true),
        Address::Contract(contract) => {
            format!(
                "{}&lt;{},{}&gt;",
                Emoji::Contract,
                contract.index,
                contract.subindex
            )
        }
    }
}

pub fn format_account_address(account: &AccountAddress, with_emoji: bool) -> String {
    let addr = account.to_string();
    let url = if is_mainnet() {
        MAINNET_API_URL
    } else {
        TESTNET_API_URL
    };

    if with_emoji {
        format!(
            r#"<a href="{}/accBalance/{}">{}{}</a>"#,
            url,
            addr,
            Emoji::Account,
            &addr[..8]
        )
    } else {
        format!(
            r#"<a href="{}/accBalance/{}">{}</a>"#,
            url,
            addr,
            &addr[..8]
        )
    }
}

pub fn format_txhash(hash: &str) -> String {
    let url = if is_mainnet() {
        MAINNET_DASHBOARD_URL
    } else {
        TESTNET_DASHBOARD_URL
    };

    format!(r#"<a href="{}/lookup/{}">{}</a>"#, url, hash, &hash[..8])
}
