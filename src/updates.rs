use crate::{db, redis_cm, sender::Message, types::*, utils::*};
use base58check::ToBase58Check;
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
        let (index_id, address, summary) = parse_payload(n11.payload());

        match serde_json::from_str(summary) {
            Ok(summary) => {
                let update = AccountUpdate {
                    index_id,
                    account: AccountAddress::new(address),
                    summary,
                };
                debug!("{:?}", update);
                handle_update(&tx, update, &mut cm).await;
            }
            Err(err) => {
                error!("{}", err);
                debug!("index_id: {} summary: {}", index_id, summary);
            }
        }
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
            for (index_id, address, summary) in updates {
                match serde_json::from_str(&summary) {
                    Ok(summary) => {
                        let update = AccountUpdate {
                            index_id,
                            account: AccountAddress::new(address),
                            summary,
                        };
                        handle_update(&tx, update, &mut cm).await;
                    }
                    Err(err) => {
                        error!("{}", err);
                        debug!("index_id: {} summary: {}", index_id, summary);
                    }
                }
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
                    | TransferWithScheduleAndMemo
                    | Update,
                ),
            result: TransactionOutcome::Success { events },
            ..
        } => match event_for(events, &update.account) {
            Some(Event::Transferred { from, to, amount }) => {
                if let Some(subscriber_ids) = db::subscriber_ids(cm, &update.account.to_string())
                    .await
                    .unwrap()
                {
                    let msg = format!(
                        "Transferred {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                        amount,
                        format_address(&from),
                        format_address(&to),
                        format_txhash(&hash),
                        format_sender(sender),
                        cost
                    );

                    tx.send(Message::new(update.index_id, subscriber_ids, msg))
                        .await
                        .ok();
                }
            }
            Some(Event::TransferredWithSchedule { from, to, amount }) => {
                if let Some(subscriber_ids) = db::subscriber_ids(cm, &update.account.to_string())
                    .await
                    .unwrap()
                {
                    let msg = format!(
                            "Transferred with schedule {} CCD from {} to {}\nTx Hash: {}\n{}Cost: {} CCD",
                            amount.total_amount(),
                            format_address(&from),
                            format_address(&to),
                            format_txhash(&hash),
                            format_sender(sender),
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

fn parse_payload(payload: &str) -> (i64, String, &str) {
    let mut triple = payload.split("|");
    let index_id: i64 = triple.next().unwrap().parse().unwrap();
    let address = hex::decode(&triple.next().unwrap()[2..])
        .unwrap()
        .to_base58check(1);
    let summary = triple.next().unwrap();
    (index_id, address, summary)
}

fn format_sender(sender: Option<AccountAddress>) -> String {
    if let Some(address) = sender {
        format!("Sender: {}\n", format_account_address(&address, false))
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
                if let Address::Account(account) = to {
                    if *address == *account {
                        return Some(event);
                    }
                }
            }
            _ => {}
        }
    }
    None
}
