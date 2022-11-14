use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

use crate::error::ContractError;
use crate::state::{ShuffleBaseIds, State};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintShuffle {
    pub nfts: ShuffleBaseIds,
    pub ticker: Option<String>,
}

#[derive(JsonSchema, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostOpenShuffle {
    /// 0 <= boost <= 1
    pub boost: f32,
    pub shuffle_price: Balance,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenShuffle {
    pub shuffle_id: String,
    pub owner: Option<String>,
    pub boost: Option<BoostOpenShuffle>,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenShuffleBatch {
    pub actions: Vec<OpenShuffle>,
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
    MintShuffle(MintShuffle),
    OpenShuffle(OpenShuffle),
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
