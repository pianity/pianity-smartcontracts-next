use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::{PackScarcity, State};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintPack {
    pub nfts: PackScarcity,
    pub ticker: Option<String>,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPack {
    pub pack_id: String,
    pub owner: Option<String>,
}

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
    MintPack(MintPack),
    OpenPack(OpenPack),
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
