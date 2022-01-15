use teloxide::utils::command::BotCommand;

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase")]
pub enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "off")]
    Help,
    #[command(description = "get current balance for an address")]
    Balance,
    #[command(description = "subscribe to on-chain events for an address")]
    Subscribe,
    #[command(description = "list subscribed addresses")]
    Subscriptions,
    #[command(description = "unsubscribe from on-chain events")]
    Unsubscribe,
}

type BotCmd = teloxide::types::BotCommand;

pub fn commands() -> [BotCmd; 5] {
    [
        BotCmd::new("help", "show help"),
        BotCmd::new("balance", "get current balance for an address"),
        BotCmd::new("subscribe", "subscribe to on-chain events for an address"),
        BotCmd::new("subscriptions", "list subscribed addresses"),
        BotCmd::new("unsubscribe", "unsubscribe from on-chain events"),
    ]
}
