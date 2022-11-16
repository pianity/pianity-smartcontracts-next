use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

use crate::error::ContractError;
use crate::state::State;

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferLocked {
    pub token_id: String,
    pub to: String,
    pub qty: Balance,
    pub duration: u32,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lock {
    pub token_id: String,
    pub qty: Balance,
    pub duration: u32,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unlock {}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configure {
    pub super_operators: Option<Vec<String>>,
    pub operators: Option<Vec<String>>,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Evolve {
    pub value: String,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Batch {
    pub actions: Vec<Action>,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "function")]
pub enum Action {
    TransferLocked(TransferLocked),
    Unlock(Unlock),
    Configure(Configure),
    Evolve(Evolve),
    Batch(Batch),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ReadResponse {
    Batch(Vec<ReadResponse>),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HandlerResult {
    Write(State),
    Read(State, ReadResponse),
}

pub type ActionResult = Result<HandlerResult, ContractError>;
