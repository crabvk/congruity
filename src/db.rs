use crate::pg_pool;
use crate::types::AccountAddress;
use base58check::ToBase58Check;
use futures::TryStreamExt;
use sqlx::{postgres::PgRow, PgPool, Row};
use std::collections::HashMap;
use std::result::Result;

/// Returns all subscriptions.
/// For each subscriber account returns a list of Telegram user IDs because many users can be subscribed to one account updates.
pub async fn all_subscriptions(pool: &PgPool) -> Result<HashMap<String, Vec<i64>>, sqlx::Error> {
    let mut subscriptions = HashMap::new();

    let mut rows =
        sqlx::query("SELECT account, array_agg(user_id) FROM subscriptions GROUP BY account")
            .fetch(pool);

    while let Some(row) = rows.try_next().await? {
        let k = row.get::<&[u8], _>(0).to_base58check(1);
        let v: Vec<i64> = row.get(1);
        subscriptions.insert(k, v);
    }

    Ok(subscriptions)
}

pub async fn subscriptions(user_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let pool = pg_pool().await;

    let subscriptions = sqlx::query("SELECT account FROM subscriptions WHERE user_id = $1")
        .bind(user_id)
        .map(|row: PgRow| row.get::<&[u8], _>(0).to_base58check(1))
        .fetch_all(pool)
        .await?;

    Ok(subscriptions)
}

pub async fn subscribe(user_id: i64, address: &AccountAddress) -> Result<Vec<i32>, sqlx::Error> {
    let pool = pg_pool().await;
    let ids = sqlx::query(
        "INSERT INTO subscriptions (user_id, account) values ($1, $2) ON CONFLICT DO NOTHING RETURNING id"
    )
        .bind(user_id)
        .bind(address.to_bytes())
        .map(|row: PgRow| row.get(0))
        .fetch_all(pool)
        .await?;

    Ok(ids)
}

pub async fn unsubscribe(
    user_id: i64,
    address: Option<&AccountAddress>,
) -> Result<Vec<i32>, sqlx::Error> {
    let pool = pg_pool().await;

    let query = if let Some(address) = address {
        sqlx::query("DELETE FROM subscriptions WHERE user_id = $1 AND account = $2 RETURNING id")
            .bind(user_id)
            .bind(address.to_bytes())
    } else {
        sqlx::query("DELETE FROM subscriptions WHERE user_id = $1 RETURNING id").bind(user_id)
    };

    let ids = query.map(|row: PgRow| row.get(0)).fetch_all(pool).await?;
    Ok(ids)
}
