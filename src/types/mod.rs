mod account_address;
mod amount;

pub use account_address::AccountAddress;
pub use amount::Amount;

use account_address::account_address_from_struct;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum BlockSummary {
    #[serde(rename = "Left", rename_all = "camelCase")]
    TransactionSummary {
        sender: Option<AccountAddress>,
        hash: String,
        cost: Amount,
        energy_cost: u64,
        r#type: TransactionSummaryType,
        result: TransactionOutcome,
        index: u64,
    },
    #[serde(rename = "Right")]
    SpecialTransactionOutcome(OutcomeKind),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "contents")]
#[serde(rename_all = "camelCase")]
pub enum TransactionSummaryType {
    AccountTransaction(TransactionType),
    CredentialDeploymentTransaction,
    UpdateTransaction,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    DeployModule,
    InitContract,
    Update,
    Transfer,
    AddBaker,
    RemoveBaker,
    UpdateBakerStake,
    UpdateBakerRestakeEarnings,
    UpdateBakerKeys,
    UpdateCredentialKeys,
    EncryptedAmountTransfer,
    TransferToEncrypted,
    TransferToPublic,
    TransferWithSchedule,
    UpdateCredentials,
    RegisterData,
    TransferWithMemo,
    EncryptedAmountTransferWithMemo,
    TransferWithScheduleAndMemo,
}

#[derive(Deserialize, Debug)]
pub struct TransactionOutcome {
    pub events: Vec<Event>,
    pub outcome: OutcomeStatus,
}

// Description of events can be found in
// https://github.com/concordium/concordium-base/blob/main/haskell-src/Concordium/Types/Execution.hs
#[derive(Deserialize, Debug)]
#[serde(tag = "tag")]
pub enum Event {
    ModuleDeployed,
    ContractInitialized,
    Updated,
    Transferred {
        #[serde(deserialize_with = "account_address_from_struct")]
        from: AccountAddress,
        #[serde(deserialize_with = "account_address_from_struct")]
        to: AccountAddress,
        amount: Amount,
    },
    AccountCreated,
    CredentialDeployed,
    BakerAdded,
    BakerRemoved,
    BakerStakeIncreased,
    BakerStakeDecreased,
    BakerSetRestakeEarnings,
    BakerKeysUpdated,
    CredentialKeysUpdated,
    NewEncryptedAmount,
    EncryptedAmountsRemoved,
    AmountAddedByDecryption,
    EncryptedSelfAmountAdded,
    UpdateEnqueued,
    TransferredWithSchedule {
        from: AccountAddress,
        to: AccountAddress,
        amount: AmountWithSchedule,
    },
    CredentialsUpdated,
    DataRegistered,
    TransferMemo {
        memo: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct AmountWithSchedule(Vec<(u64, String)>);

impl AmountWithSchedule {
    pub fn total_amount(&self) -> Amount {
        self.0
            .iter()
            .fold(0, |acc, (_, amount)| acc + amount.parse::<u64>().unwrap())
            .into()
    }
}

#[derive(Deserialize, Debug)]
pub struct TransferMemo {
    pub memo: String,
}

#[derive(Deserialize, Debug)]
pub struct ContractAddress {
    pub index: u64,
    pub subindex: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OutcomeStatus {
    Success,
    Reject,
}

// Description can be found in
// https://github.com/concordium/concordium-base/blob/main/haskell-src/Concordium/Types/Transactions.hs
#[derive(Deserialize, Debug)]
#[serde(tag = "tag")]
pub enum OutcomeKind {
    BakingRewards,
    Mint,
    #[serde(rename_all = "camelCase")]
    FinalizationRewards {
        finalization_rewards: Vec<AccountAmount>,
    },
    BlockReward,
}

#[derive(Deserialize, Debug)]
pub struct AccountAmount {
    pub address: String,
    pub amount: Amount,
}
