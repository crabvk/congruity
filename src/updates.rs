use crate::{db, pg_pool, redis_client};
use crate::{types::*, utils::*, BotType};
use log::*;
use redis::{aio::Connection, AsyncCommands};
use sqlx::{postgres::PgListener, Pool, Postgres};
use std::collections::HashMap;
use teloxide::prelude::*;

const TX_CHANNEL: &str = "tx_channel";

pub async fn handle_updates(bot: BotType) -> Result<(), sqlx::Error> {
    let mut conn = redis_client().await.get_async_connection().await.unwrap();

    let pool = pg_pool().await;
    let mut listener = PgListener::connect(&env("POSTGRESQL_URL")).await?;

    info!("Listening channel {}", TX_CHANNEL);
    listener.listen(TX_CHANNEL).await?;

    loop {
        let n11 = listener.recv().await?;
        let json = n11.payload();
        let update: AccountUpdate = serde_json::from_str(json).unwrap();
        debug!("{:?}", update);
        let subscriptions = db::all_subscriptions(pool).await?;
        handle_update(&bot, &mut conn, update, &subscriptions).await;
    }
}

/// Processes account updates since last handled account transaction index.
pub async fn process_updates_since_last_ati(bot: &BotType, pool: &Pool<Postgres>) {
    let mut conn = redis_client().await.get_async_connection().await.unwrap();
    let index_id: Option<i64> = conn.get("ati:latest").await.unwrap();

    if let Some(index_id) = index_id {
        info!("Last account transaction index ID={}", index_id);
        let updates = db::account_updates_since(index_id).await.unwrap();

        if updates.len() > 0 {
            info!("Processing {} account updates", updates.len());
            let subscriptions = db::all_subscriptions(pool).await.unwrap();
            for update in updates {
                handle_update(&bot, &mut conn, update, &subscriptions).await;
            }
        } else {
            info!("No account updates found");
        }
    } else {
        info!("Last account transaction index ID not found");
    }
}

async fn handle_update(
    bot: &BotType,
    conn: &mut Connection,
    update: AccountUpdate,
    subscriptions: &HashMap<String, Vec<i64>>,
) {
    use TransactionType::*;

    match update.summary {
        BlockSummary::TransactionSummary {
            sender,
            hash,
            cost,
            r#type:
                TransactionSummaryType::AccountTransaction(
                    Transfer
                    | TransferWithMemo
                    | TransferWithSchedule
                    | TransferWithScheduleAndMemo,
                ),
            result:
                TransactionOutcome {
                    events,
                    outcome: OutcomeStatus::Success,
                },
            ..
        } => match event_for(events, &update.account) {
            Some(Event::Transferred { from, to, amount }) => {
                if let Some(subscriber_ids) = subscriptions.get(&to.to_string()) {
                    let msg = format!(
                        "Transferred {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                        amount,
                        address_to_hyperlink(&from, Some(Emoji::Person)),
                        address_to_hyperlink(&to, Some(Emoji::Person)),
                        txhash_to_hyperlink(&hash),
                        sender_hyperlink(sender),
                        cost
                    );

                    notify_subscribers(&bot, msg, subscriber_ids).await;
                    set_index_id(conn, update.index_id).await;
                }
            }
            Some(Event::TransferredWithSchedule { from, to, amount }) => {
                if let Some(subscriber_ids) = subscriptions.get(&to.to_string()) {
                    let msg = format!(
                            "Transferred with schedule {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                            amount.total_amount(),
                            address_to_hyperlink(&from, Some(Emoji::Person)),
                            address_to_hyperlink(&to, Some(Emoji::Person)),
                            txhash_to_hyperlink(&hash),
                            sender_hyperlink(sender),
                            cost
                        );

                    notify_subscribers(&bot, msg, subscriber_ids).await;
                    set_index_id(conn, update.index_id).await;
                }
            }
            _ => {}
        },
        BlockSummary::SpecialTransactionOutcome(OutcomeKind::BakingRewards {
            ref baker_rewards,
        }) => {
            let reward = baker_rewards
                .iter()
                .find(|r| r.address == update.account.address());

            if let Some(reward) = reward {
                if let Some(subscriber_ids) = subscriptions.get(&reward.address.to_string()) {
                    let msg = format!("Baker reward {} CCD", reward.amount,);
                    notify_subscribers(&bot, msg, subscriber_ids).await;
                    set_index_id(conn, update.index_id).await;
                }
            }
        }
        _ => {}
    }
}

fn sender_hyperlink(sender: Option<AccountAddress>) -> String {
    if let Some(address) = sender {
        format!("Sender: {}\n", address_to_hyperlink(&address, None))
    } else {
        String::new()
    }
}

async fn set_index_id(conn: &mut Connection, index_id: i64) {
    let _: () = conn.set("ati:latest", index_id).await.unwrap();
}

fn event_for(events: Vec<Event>, address: &AccountAddress) -> Option<Event> {
    use Event::*;

    let mut iter = events.into_iter();
    while let Some(event) = iter.next() {
        match event {
            Transferred { ref to, .. } | TransferredWithSchedule { ref to, .. } => {
                if *address == *to {
                    return Some(event);
                }
            }
            _ => {}
        }
    }
    None
}

async fn notify_subscribers(bot: &BotType, message: String, subscriber_ids: &Vec<i64>) {
    for sid in subscriber_ids {
        bot.send_message(*sid, &message).await.ok();
    }
}
