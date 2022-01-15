# Telegram bot for Concordium blockchain

Get notifications about on-chain events.

> Bot is currently running in mainnet [@CCDMainnetBot](https://t.me/CCDMainnetBot) and testnet [@CCDTestnetBot](https://t.me/CCDTestnetBot).

## Supported events

* Transfer
* TransferWithSchedule
* BakingRewards

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

* recover from crash: query missed events on startup
* handle crashes in spawned tasks
* write more code comments
* write tests

## Contributions

Contributions are very welcome.  
Feel free to create an issue if you found a bug, want to request a feature or have a question.

## Sponsoring

Work on this project is done in my free time and takes time and effort.  
If you find the bot useful and want to help its depelopment please donate some CCD to 49YugmeWQGApKgpzkR7rcefA7KaoujeToXkA8umdCvJ1DFq4RN.

## Resources

* [Mainnet documentation](https://developer.concordium.software/en/mainnet/net/index.html)
* [Running a node with finalized transaction logging](https://github.com/Concordium/concordium-node/blob/main/docs/transaction-logging.md)
* [Transaction execution events](https://github.com/concordium/concordium-base/blob/main/haskell-src/Concordium/Types/Execution.hs)
* [Special transaction outcomes](https://github.com/concordium/concordium-base/blob/main/haskell-src/Concordium/Types/Transactions.hs)
