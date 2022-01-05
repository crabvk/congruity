# Telegram bot for Concordium blockchain

Get notifications about on-chain events.

> Check it out! Bot is currently running in testnet [@CCDTestnetBot](https://t.me/CCDTestnetBot).

## Installation

Clone repo with submodules

```shell
git clone --recurse-submodules https://github.com/crabvk/congruity.git
```

and compile project with

```shell
cargo build --locked --release
```

## Configuration

The bot is configured via dotenv file. See example with option descriptions in [.example.env](/.example.env).  
When the application starts, `.env` file is loaded from the current directory or any of its parents.

## TODO

* handle crashes in spawned tasks
* write code comments
* write tests

## Resources

* [Mainnet documentation](https://developer.concordium.software/en/mainnet/net/index.html)
* [Running a node with finalized transaction logging](https://github.com/Concordium/concordium-node/blob/main/docs/transaction-logging.md)
