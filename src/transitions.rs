use super::states::*;
use crate::types::{AccountAddress, Amount};
use crate::{command::Command, db, rpc, states::Dialogue, BotType};
use log::*;
use teloxide::payloads::SendMessageSetters;
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove};
use teloxide::utils::command::BotCommand;
use teloxide::{prelude::*, requests::ResponseResult};

const BOT_NAME: &str = "Congruity";

async fn subscribe(address: &AccountAddress, cx: TransitionIn<BotType>) -> ResponseResult<Message> {
    let user_id = cx.chat_id() as i64;
    let result = db::subscribe(user_id, address).await;
    match result {
        Ok(true) => cx.answer("Subscribed successfully").await,
        Ok(false) => {
            cx.answer("You're already subscribed for this address")
                .await
        }
        Err(err) => cx.answer(format!("{}", err)).await,
    }
}

async fn answer_after_keyboard(cx: TransitionIn<BotType>, text: &str) -> ResponseResult<Message> {
    cx.requester
        .send_message(cx.chat_id(), text)
        .reply_markup(KeyboardRemove::new())
        .await
}

async fn unsubscribe_all(cx: TransitionIn<BotType>) -> ResponseResult<Message> {
    match db::unsubscribe_all(cx.chat_id()).await {
        Ok(true) => answer_after_keyboard(cx, "Unsubscribed successfully").await,
        Ok(false) => answer_after_keyboard(cx, "No subscriptions were found").await,
        Err(err) => {
            error!("{}", err);
            answer_after_keyboard(cx, "A database query error has occurred 😐").await
        }
    }
}

async fn unsubscribe(
    address: &AccountAddress,
    cx: TransitionIn<BotType>,
) -> ResponseResult<Message> {
    match db::unsubscribe(cx.chat_id(), address).await {
        Ok(true) => answer_after_keyboard(cx, "Unsubscribed successfully").await,
        Ok(false) => answer_after_keyboard(cx, "You're not subscribed for this address").await,
        Err(err) => {
            error!("{}", err);
            answer_after_keyboard(cx, "A database query error has occurred 😐").await
        }
    }
}

async fn get_account_balance(
    addr: &AccountAddress,
    cx: TransitionIn<BotType>,
) -> ResponseResult<Message> {
    match rpc::get_account_balance(&addr.to_string()).await {
        Ok(Some(amount)) => {
            let amount: Amount = amount.parse().unwrap();
            let answer = format!("{} CCD", amount.to_string());
            cx.answer(answer).await
        }
        Ok(None) => cx.answer("Account address not found").await,
        Err(err) => {
            let msg = format!("Error: {}", err);
            cx.answer(msg).await
        }
    }
}

fn build_keyboard(buttons: &[String]) -> KeyboardMarkup {
    let buttons = buttons.iter().map(|text| {
        [KeyboardButton {
            text: text.to_string(),
            request: None,
        }]
    });

    KeyboardMarkup::new(buttons)
        .resize_keyboard(true)
        .one_time_keyboard(true)
}

#[teloxide(subtransition)]
async fn start(
    state: StartState,
    cx: TransitionIn<BotType>,
    text: String,
) -> TransitionOut<Dialogue> {
    if text.starts_with("/") {
        let command = match Command::parse(&text, BOT_NAME) {
            Ok(cmd) => cmd,
            Err(err) => {
                cx.answer(err.to_string()).await?;
                return next(state);
            }
        };

        match command {
            Command::Start | Command::Help => {
                cx.answer(Command::descriptions()).await?;
            }
            Command::Balance => {
                cx.answer("OK, send me address of the account").await?;
                return next(ReceiveAddressState::Balance);
            }
            Command::Subscribe => {
                cx.answer("OK, send me address of the account").await?;
                return next(ReceiveAddressState::Subscribe);
            }
            Command::Subscriptions => {
                let user_id = cx.chat_id() as i64;
                let subscriptions = db::subscriptions(user_id).await.unwrap();

                if subscriptions.len() > 0 {
                    cx.answer(subscriptions.join("\n")).await?;
                } else {
                    cx.answer("No subscriptions were found").await?;
                }
            }
            Command::Unsubscribe => {
                let user_id = cx.chat_id() as i64;
                let mut subscriptions = db::subscriptions(user_id).await.unwrap();

                if subscriptions.len() > 0 {
                    if subscriptions.len() > 1 {
                        subscriptions.push("all".to_string());
                    }
                    let keyboard = build_keyboard(&subscriptions);
                    cx.requester
                        .send_message(user_id, "OK, send me address of the account")
                        .reply_markup(keyboard)
                        .await?;
                    return next(ReceiveAddressState::Unsubscribe);
                } else {
                    cx.answer("No subscriptions were found").await?;
                }
            }
        }
    } else {
        cx.answer("Don't understand 🤷‍♂️").await?;
    };

    next(state)
}

#[teloxide(subtransition)]
async fn recieve_balance(
    state: ReceiveAddressState,
    cx: TransitionIn<BotType>,
    address: String,
) -> TransitionOut<Dialogue> {
    use ReceiveAddressState::*;

    if state == Unsubscribe && address == "all" {
        unsubscribe_all(cx).await?;
        return next(StartState);
    }

    if let Ok(address) = address.parse::<AccountAddress>() {
        debug!("{:?} {}", state, address);
        match state {
            Balance => {
                get_account_balance(&address, cx).await?;
            }
            Subscribe => {
                subscribe(&address, cx).await?;
            }
            Unsubscribe => {
                unsubscribe(&address, cx).await?;
            }
        };
    } else {
        cx.answer("Invalid account address").await?;
    }
    next(StartState)
}
