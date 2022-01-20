use derive_more::From;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use teloxide::dispatching::dialogue::Transition;

#[derive(Transition, From, Serialize, Deserialize)]
pub enum Dialogue {
    Start(StartState),
    ReceiveAddress(ReceiveAddressState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}

#[derive(Serialize, Deserialize)]
pub struct StartState;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ReceiveAddressState {
    Balance,
    Subscribe,
    Unsubscribe,
}
