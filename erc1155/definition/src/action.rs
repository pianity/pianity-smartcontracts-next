use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::{Balance, BalancePrecision, Parameters, Settings, Token};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Initialize;

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BalanceOf {
    pub token_id: Option<String>,
    pub target: String,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetToken {
    pub token_id: Option<String>,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadSettings;

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
    pub can_evolve: Option<bool>,
    pub allow_free_transfer: Option<bool>,
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
    Initialize(Initialize),
    BalanceOf(BalanceOf),
    GetToken(GetToken),
    ReadSettings(ReadSettings),
    Transfer(Transfer),
    Configure(Configure),
    SetApprovalForAll(SetApprovalForAll),
    IsApprovedForAll(IsApprovedForAll),
    Evolve(Evolve),
    Mint(Mint),
    Burn(Burn),
    Batch(Batch),
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ReadResponse {
    BalanceOf {
        balance: Balance,
        target: String,
    },

    GetToken(Token),

    ReadSettings(Settings),

    IsApprovedForAll {
        approved: bool,
        owner: String,
        operator: String,
    },

    Batch(Vec<ReadResponse>),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HandlerResult {
    Write(Parameters),
    Read(Parameters, ReadResponse),
    None(Parameters),
}

pub type ActionResult = Result<HandlerResult, ContractError>;
