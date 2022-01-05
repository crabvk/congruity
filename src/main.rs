mod command;
mod db;
mod events;
mod listener;
mod repl;
mod rpc;
mod states;
mod transitions;
mod types;
mod utils;

use listener::webhook;
use log::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use teloxide::{
    adaptors::DefaultParseMode, dispatching::update_listeners, prelude::*, types::ParseMode,
};
use tokio::sync::OnceCell;
use utils::*;

type BotType = AutoSend<DefaultParseMode<Bot>>;

static PG_POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();

pub async fn pg_pool() -> &'static Pool<Postgres> {
    PG_POOL
        .get_or_init(|| async {
            PgPoolOptions::new()
                .max_connections(16)
                .connect_lazy(&env("POSTGRESQL_URL"))
                .unwrap()
        })
        .await
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    dotenv::dotenv().ok();
    teloxide::enable_logging!();

    let pool = pg_pool().await;
    sqlx::migrate!().run(pool).await.unwrap();

    let token = env("TELEGRAM_TOKEN");
    let bot = Bot::new(&token).parse_mode(ParseMode::Html).auto_send();

    // Update the list of the bot's commands
    bot.set_my_commands(command::commands()).await.unwrap();

    let cloned_bot = bot.clone();

    // Handle on-chain events via PostgreSQL pub/sub channel
    tokio::spawn(events::handle_events(cloned_bot));

    if let Some(host) = std::env::var("TELEGRAM_WEBHOOK_HOST").ok() {
        info!("Receiving updates via webhook on host: {}", host);
        let listener = webhook(host, token, &bot).await;
        repl::dialogue_repl(bot, listener).await;
    } else {
        info!("Using long polling to fetch updates");
        let listener = update_listeners::polling_default(bot.clone()).await;
        repl::dialogue_repl(bot, listener).await;
    };
}
