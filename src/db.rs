use crate::types::{AccountAddress, AccountUpdate};
use crate::{pg_pool, redis_cm};
use base58check::ToBase58Check;
use redis::{aio::ConnectionManager, AsyncCommands, RedisResult};
use sqlx::{postgres::PgRow, PgPool, Row};
use std::result::Result;
use tokio_stream::StreamExt;

/// Preloads all subscriptions from Postgres to Redis.
/// For each subscriber account there are a set of Telegram user IDs.
/// Many users can subscribe to updates for one account.
pub async fn load_subscriptions(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut cm = redis_cm().await.clone();
    let account_keys: Vec<String> = cm.keys("account:*").await.unwrap();
    for key in account_keys {
        let _: () = cm.del(key).await.unwrap();
    }

    let mut rows =
        sqlx::query("SELECT account, array_agg(user_id) FROM subscriptions GROUP BY account")
            .fetch(pool);

    while let Some(row) = rows.try_next().await? {
        let k = row.get::<&[u8], _>(0).to_base58check(1);
        let v: Vec<i64> = row.get(1);
        let key = format!("account:{}", k);
        let _: () = cm.sadd(key, v).await.unwrap();
    }

    Ok(())
}

pub async fn subscriber_ids(
    cm: &mut ConnectionManager,
    account: &str,
) -> RedisResult<Option<Vec<i64>>> {
    let key = format!("account:{}", account);
    let user_ids: Option<Vec<i64>> = cm.smembers(key).await?;
    Ok(user_ids)
}

/// Returns subscriptions for a Telegram user.
pub async fn subscriptions(user_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let pool = pg_pool().await;

    let subscriptions = sqlx::query("SELECT account FROM subscriptions WHERE user_id = $1")
        .bind(user_id)
        .map(|row: PgRow| row.get::<&[u8], _>(0).to_base58check(1))
        .fetch_all(pool)
        .await?;

    Ok(subscriptions)
}

pub async fn subscribe(user_id: i64, address: &AccountAddress) -> Result<bool, sqlx::Error> {
    let mut cm = redis_cm().await.clone();
    let pool = pg_pool().await;
    let ids: Vec<i64> = sqlx::query(
        r#"
INSERT INTO subscriptions (user_id, account) VALUES ($1, $2)
ON CONFLICT DO NOTHING RETURNING user_id
        "#,
    )
    .bind(user_id)
    .bind(address.to_bytes())
    .map(|row: PgRow| row.get(0))
    .fetch_all(pool)
    .await?;

    if ids.len() > 0 {
        let key = format!("account:{}", address);
        let _: () = cm.sadd(key, &ids).await.unwrap();
    }

    Ok(ids.len() > 0)
}

pub async fn unsubscribe(user_id: i64, address: &AccountAddress) -> Result<bool, sqlx::Error> {
    let mut cm = redis_cm().await.clone();
    let pool = pg_pool().await;

    let ids: Vec<i64> = sqlx::query(
        "DELETE FROM subscriptions WHERE user_id = $1 AND account = $2 RETURNING user_id",
    )
    .bind(user_id)
    .bind(address.to_bytes())
    .map(|row: PgRow| row.get(0))
    .fetch_all(pool)
    .await?;

    if ids.len() > 0 {
        let key = format!("account:{}", address);
        let _: () = cm.srem(key, &ids).await.unwrap();
    }

    Ok(ids.len() > 0)
}

pub async fn unsubscribe_all(user_id: i64) -> Result<bool, sqlx::Error> {
    let mut cm = redis_cm().await.clone();
    let pool = pg_pool().await;

    let pairs: Vec<(i64, String)> =
        sqlx::query("DELETE FROM subscriptions WHERE user_id = $1 RETURNING user_id, account")
            .bind(user_id)
            .map(|row: PgRow| (row.get(0), row.get::<&[u8], _>(1).to_base58check(1)))
            .fetch_all(pool)
            .await?;

    for (user_id, account) in &pairs {
        let key = format!("account:{}", account);
        let _: () = cm.srem(key, user_id).await.unwrap();
    }

    Ok(pairs.len() > 0)
}

/// Returns account updates since account transaction index ID.
pub async fn account_updates_since(index_id: i64) -> Result<Vec<AccountUpdate>, sqlx::Error> {
    let pool = pg_pool().await;

    let updates = sqlx::query(
        r#"
SELECT ati.id, ati.account, sm.summary::text FROM ati
JOIN summaries AS sm ON ati.summary = sm.id
WHERE ati.id > $1 AND ati.account IN (SELECT DISTINCT ON (account) account FROM subscriptions)
        "#,
    )
    .bind(index_id)
    .map(|row: PgRow| AccountUpdate::new(row.get(0), row.get(1), row.get(2)))
    .fetch_all(pool)
    .await?;

    Ok(updates)
}
