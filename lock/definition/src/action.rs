use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

use crate::error::ContractError;
use crate::state::{LockedBalance, Parameters};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Initialize;

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetVault {
    pub owner: String,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetAllVaults;

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReleaseMethod {
    Cliff,
    Linear,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferLocked {
    pub token_id: String,
    pub target: String,
    pub qty: Balance,
    pub duration: u32,
    pub method: ReleaseMethod,
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

#[derive(JsonSchema, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Configure {
    pub paused: Option<bool>,
    pub can_evolve: Option<bool>,
    pub super_operators: Option<Vec<String>>,
    pub operators: Option<Vec<String>>,
    pub erc1155: Option<String>,
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
    Initialize(Initialize),
    GetVault(GetVault),
    GetAllVaults(GetAllVaults),
    TransferLocked(TransferLocked),
    Unlock(Unlock),
    Configure(Configure),
    Evolve(Evolve),
    Batch(Batch),
}

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ReadResponse {
    Batch(Vec<ReadResponse>),
    GetVault((String, Vec<LockedBalance>)),
    GetAllVaults(Vec<(String, Vec<LockedBalance>)>),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HandlerResult {
    Write(Parameters),
    Read(Parameters, ReadResponse),
    None(Parameters),
}

pub type ActionResult = Result<HandlerResult, ContractError>;
