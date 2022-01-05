use crate::states::Dialogue;
use crate::utils::env;
use crate::BotType;
use teloxide::{
    dispatching::dialogue::{serializer::Json, RedisStorage, Storage},
    dispatching::update_listeners::UpdateListener,
    prelude::*,
    RequestError,
};
use thiserror::Error;

type StorageError = <RedisStorage<Json> as Storage<Dialogue>>::Error;
type In = DialogueWithCx<BotType, Message, Dialogue, StorageError>;

#[derive(Debug, Error)]
enum Error {
    #[error("error from Telegram: {0}")]
    TelegramError(#[from] RequestError),
    #[error("error from storage: {0}")]
    StorageError(#[from] StorageError),
}

pub async fn dialogue_repl<UListener, ListenerE>(bot: BotType, listener: UListener)
where
    UListener: UpdateListener<ListenerE>,
    ListenerE: std::fmt::Debug,
{
    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::with_storage(
            |DialogueWithCx { cx, dialogue }: In| async move {
                let dialogue = dialogue.expect("std::convert::Infallible");
                handle_message(cx, dialogue)
                    .await
                    .expect("Something wrong with the bot!")
            },
            RedisStorage::open(env("REDIS_URL"), Json).await.unwrap(),
        ))
        .setup_ctrlc_handler()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}

async fn handle_message(
    cx: UpdateWithCx<BotType, Message>,
    dialogue: Dialogue,
) -> TransitionOut<Dialogue> {
    match cx.update.text().map(ToOwned::to_owned) {
        None => {
            cx.answer("Send me a command").await?;
            next(dialogue)
        }
        Some(ans) => dialogue.react(cx, ans).await,
    }
}
