use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::{Balance, BalancePrecision, State};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BalanceOf {
    pub token_id: Option<String>,
    pub target: String,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub from: Option<String>,
    pub to: String,
    pub token_id: Option<String>,
    pub qty: Balance,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Configure {
    pub super_operators: Option<Vec<String>>,
    pub operators: Option<Vec<String>>,
    pub proxies: Option<Vec<String>>,
    pub paused: Option<bool>,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Mint {
    pub base_id: Option<String>,
    pub prefix: Option<String>,
    pub qty: Balance,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Burn {
    pub token_id: Option<String>,
    pub qty: Balance,
    pub owner: Option<String>,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetApprovalForAll {
    pub operator: String,
    pub approved: bool,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IsApprovedForAll {
    pub owner: String,
    pub operator: String,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Batch {
    pub actions: Vec<Action>,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Evolve {
    pub value: String,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "function")]
pub enum Action {
    BalanceOf(BalanceOf),
    Transfer(Transfer),
    Configure(Configure),
    SetApprovalForAll(SetApprovalForAll),
    IsApprovedForAll(IsApprovedForAll),
    Evolve(Evolve),
    Mint(Mint),
    Burn(Burn),
    Batch(Batch),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
