use crate::{db, redis_cm, sender::Message, types::*, utils::*};
use log::*;
use redis::{aio::ConnectionManager, AsyncCommands};
use sqlx::postgres::PgListener;
use tokio::sync::mpsc::Sender;

const TX_CHANNEL: &str = "tx_channel";

pub async fn handle_updates(tx: Sender<Message>) -> Result<(), sqlx::Error> {
    let mut cm = redis_cm().await.clone();
    let mut listener = PgListener::connect(&env("POSTGRESQL_URL")).await?;

    info!("Listening channel {}", TX_CHANNEL);
    listener.listen(TX_CHANNEL).await?;

    loop {
        let n11 = listener.recv().await?;
        let json = n11.payload();
        let update: AccountUpdate = serde_json::from_str(json).unwrap();
        debug!("{:?}", update);
        handle_update(&tx, update, &mut cm).await;
    }
}

/// Processes account updates since last handled account transaction index.
pub async fn process_updates_since_last_ati(tx: Sender<Message>) {
    let mut cm = redis_cm().await.clone();
    let index_id: Option<i64> = cm.get("ati:latest").await.unwrap();

    if let Some(index_id) = index_id {
        info!("Last account transaction index ID {}", index_id);
        let updates = db::account_updates_since(index_id).await.unwrap();

        if updates.len() > 0 {
            info!("Processing {} account updates", updates.len());
            for update in updates {
                handle_update(&tx, update, &mut cm).await;
            }
        } else {
            info!("No account updates found");
        }
    } else {
        info!("Last account transaction index ID not found");
    }
}

/// Handles update for account.
async fn handle_update(tx: &Sender<Message>, update: AccountUpdate, cm: &mut ConnectionManager) {
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
                if let Some(subscriber_ids) = db::subscriber_ids(cm, &to.to_string()).await.unwrap()
                {
                    let msg = format!(
                        "Transferred {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                        amount,
                        address_to_hyperlink(&from, Some(Emoji::Person)),
                        address_to_hyperlink(&to, Some(Emoji::Person)),
                        txhash_to_hyperlink(&hash),
                        sender_hyperlink(sender),
                        cost
                    );

                    tx.send(Message::new(update.index_id, subscriber_ids, msg))
                        .await
                        .ok();
                }
            }
            Some(Event::TransferredWithSchedule { from, to, amount }) => {
                if let Some(subscriber_ids) = db::subscriber_ids(cm, &to.to_string()).await.unwrap()
                {
                    let msg = format!(
                            "Transferred with schedule {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                            amount.total_amount(),
                            address_to_hyperlink(&from, Some(Emoji::Person)),
                            address_to_hyperlink(&to, Some(Emoji::Person)),
                            txhash_to_hyperlink(&hash),
                            sender_hyperlink(sender),
                            cost
                        );

                    tx.send(Message::new(update.index_id, subscriber_ids.to_vec(), msg))
                        .await
                        .ok();
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
                if let Some(subscriber_ids) = db::subscriber_ids(cm, &reward.address).await.unwrap()
                {
                    let msg = format!("Baker reward {} CCD", reward.amount,);
                    tx.send(Message::new(update.index_id, subscriber_ids.to_vec(), msg))
                        .await
                        .ok();
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
