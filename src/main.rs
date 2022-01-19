mod command;
mod db;
mod listener;
mod repl;
mod rpc;
mod sender;
mod states;
mod transitions;
mod types;
mod updates;
mod utils;

use listener::webhook;
use log::*;
use redis::{aio::ConnectionManager, Client};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use teloxide::{
    adaptors::DefaultParseMode, dispatching::update_listeners, prelude::*, types::ParseMode,
};
use tokio::sync::{mpsc, OnceCell};
use utils::*;

type BotType = AutoSend<DefaultParseMode<Bot>>;

static PG_POOL: OnceCell<Pool<Postgres>> = OnceCell::const_new();
static REDIS: OnceCell<ConnectionManager> = OnceCell::const_new();

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

pub async fn redis_cm() -> &'static ConnectionManager {
    REDIS
        .get_or_init(|| async {
            Client::open(env("REDIS_URL"))
                .unwrap()
                .get_tokio_connection_manager()
                .await
                .expect("Can't create Redis connection manager")
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

    // Update the list of the bot commands
    bot.set_my_commands(command::commands()).await.unwrap();

    // Spawn Telegram messages sender
    let (tx, rx) = mpsc::channel(2048);
    tokio::spawn(sender::handle_messages(rx, bot.clone()));

    // Find and process missing account updates
    updates::process_updates_since_last_ati(tx.clone(), pool).await;

    // Handle Concordium account updates via PostgreSQL pub/sub channel
    tokio::spawn(updates::handle_updates(tx));

    if let Some(host) = std::env::var("TELEGRAM_WEBHOOK_HOST").ok() {
        info!("Receiving updates via webhook: {}", host);
        let listener = webhook(host, token, &bot).await;
        repl::dialogue_repl(bot, listener).await;
    } else {
        info!("Using long polling to fetch updates");
        let listener = update_listeners::polling_default(bot.clone()).await;
        repl::dialogue_repl(bot, listener).await;
    };
}
