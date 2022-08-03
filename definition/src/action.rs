use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::{Balance, BalancePrecision, State};

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceOf {
    pub token_id: String,
    pub target: String,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub from: Option<String>,
    pub to: String,
    pub token_id: String,
    pub qty: Balance,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configure {
    pub super_owner: Option<String>,
    pub owners: Option<Vec<String>>,
    pub authorized_addresses: Option<Vec<String>>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mint {
    pub ticker: Option<String>,
    pub prefix: Option<String>,
    pub qty: Balance,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetApprovalForAll {
    pub operator: String,
    pub approved: bool,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsApprovedForAll {
    pub owner: String,
    pub operator: String,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Batch {
    pub actions: Vec<Action>,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evolve {
    pub value: String,
}

#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "function")]
pub enum Action {
    #[serde(rename_all = "camelCase")]
    BalanceOf(BalanceOf),

    #[serde(rename_all = "camelCase")]
    Transfer(Transfer),

    Configure(Configure),

    SetApprovalForAll(SetApprovalForAll),

    IsApprovedForAll(IsApprovedForAll),

    Evolve(Evolve),

    Mint(Mint),

    Batch(Batch),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ReadResponse {
    Balance {
        balance: BalancePrecision,
        target: String,
    },

    ApprovedForAll {
        approved: bool,
        owner: String,
        operator: String,
    },

    Batch(Vec<ReadResponse>),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HandlerResult {
    Write(State),
    Read(State, ReadResponse),
}

pub type ActionResult = Result<HandlerResult, ContractError>;
