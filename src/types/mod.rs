mod account_address;
mod amount;

pub use account_address::AccountAddress;
pub use amount::Amount;
use serde::Deserialize;

#[derive(Debug)]
pub struct AccountUpdate {
    pub index_id: i64,
    pub account: AccountAddress,
    pub summary: BlockSummary,
}

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
#[serde(tag = "type", content = "contents", rename_all = "camelCase")]
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
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum TransactionOutcome {
    Success { events: Vec<Event> },
    Reject,
}

/// Transaction execution events.
/// More info: https://git.io/J9cQA
#[derive(Deserialize, Debug)]
#[serde(tag = "tag")]
pub enum Event {
    ModuleDeployed,
    ContractInitialized,
    Updated,
    Transferred {
        from: Address,
        to: Address,
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
        from: Address,
        to: Address,
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
#[serde(tag = "type", content = "address")]
pub enum Address {
    #[serde(rename = "AddressAccount")]
    Account(AccountAddress),
    #[serde(rename = "AddressContract")]
    Contract(ContractAddress),
}

/// Special transaction outcomes.
/// More info: https://git.io/J0thA
#[derive(Deserialize, Debug)]
#[serde(tag = "tag")]
pub enum OutcomeKind {
    #[serde(rename_all = "camelCase")]
    BakingRewards {
        baker_rewards: Vec<AddressWithAmount>,
    },
    Mint,
    FinalizationRewards,
    BlockReward,
}

#[derive(Deserialize, Debug)]
pub struct AddressWithAmount {
    pub address: String,
    pub amount: Amount,
}
