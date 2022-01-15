use crate::{db, pg_pool};
use crate::{types::*, utils::*, BotType};
use log::*;
use sqlx::postgres::PgListener;
use teloxide::prelude::*;

const TX_CHANNEL: &str = "tx_channel";

pub async fn handle_events(bot: BotType) -> Result<(), sqlx::Error> {
    use TransactionType::*;

    let pool = pg_pool().await;
    let mut listener = PgListener::connect(&env("POSTGRESQL_URL")).await?;

    info!("Listening channel {}", TX_CHANNEL);
    listener.listen(TX_CHANNEL).await?;

    loop {
        let n11 = listener.recv().await?;
        let json = n11.payload();
        debug!("{}", json);
        let block: BlockSummary = serde_json::from_str(json).unwrap();
        let subscriptions = db::all_subscriptions(pool).await?;
        debug!("{:?}", block);

        match block {
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
            } => {
                let sender = if let Some(address) = sender {
                    format!("Sender: {}\n", address_to_hyperlink(&address, None))
                } else {
                    String::new()
                };
                let mut iter = events.iter();
                while let Some(event) = iter.next() {
                    match event {
                        Event::Transferred { from, to, amount } => {
                            if let Some(subscriber_ids) = subscriptions.get(&to.to_string()) {
                                let msg = format!(
                                    "Transferred {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                                    amount,
                                    address_to_hyperlink(&from, Some(Emoji::Person)),
                                    address_to_hyperlink(&to, Some(Emoji::Person)),
                                    txhash_to_hyperlink(&hash),
                                    sender,
                                    cost
                                );

                                notify_subscribers(&bot, msg, subscriber_ids).await
                            }
                        }
                        Event::TransferredWithSchedule { from, to, amount } => {
                            if let Some(subscriber_ids) = subscriptions.get(&to.to_string()) {
                                let msg = format!(
                                    "Transferred with schedule {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                                    amount.total_amount(),
                                    address_to_hyperlink(&from, Some(Emoji::Person)),
                                    address_to_hyperlink(&to, Some(Emoji::Person)),
                                    txhash_to_hyperlink(&hash),
                                    sender,
                                    cost
                                );

                                notify_subscribers(&bot, msg, subscriber_ids).await
                            }
                        }
                        _ => {}
                    }
                }
            }
            BlockSummary::SpecialTransactionOutcome(OutcomeKind::BakingRewards {
                baker_rewards,
            }) => {
                for reward in baker_rewards {
                    if let Some(subscriber_ids) = subscriptions.get(&reward.address.to_string()) {
                        let msg = format!("Baker reward {} CCD", reward.amount,);
                        notify_subscribers(&bot, msg, subscriber_ids).await
                    }
                }
            }
            _ => {}
        }
    }
}

async fn notify_subscribers(bot: &BotType, message: String, subscriber_ids: &Vec<i64>) {
    for sid in subscriber_ids {
        bot.send_message(*sid, &message).await.ok();
    }
}
