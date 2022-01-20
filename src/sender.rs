use crate::{redis_cm, BotType};
use log::*;
use redis::AsyncCommands;
use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct Message {
    index_id: i64,
    user_ids: Vec<i64>,
    text: String,
}

impl Message {
    pub fn new(index_id: i64, user_ids: Vec<i64>, text: String) -> Self {
        Self {
            index_id,
            user_ids,
            text,
        }
    }
}

pub async fn handle_messages(mut rx: Receiver<Message>, bot: BotType) {
    let mut conn = redis_cm().await.clone();

    while let Some(msg) = rx.recv().await {
        for user_id in msg.user_ids {
            match bot.send_message(user_id, &msg.text).await {
                Ok(_) => debug!("Message sent to Telegram ID {}", user_id),
                Err(err) => error!("{}", err),
            }
        }
        let _: () = conn.set("ati:latest", msg.index_id).await.unwrap();
    }
}
