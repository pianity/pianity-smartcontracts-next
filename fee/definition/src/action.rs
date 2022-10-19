use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

use crate::error::ContractError;
use crate::state::{Fees, State};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    // pub from: Option<String>,
    pub to: String,
    pub nft_id: String,
    pub price: Balance,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFee {
    pub nft_base_id: String,
    pub fees: Fees,
    pub rate: u32,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configure {
    pub super_operator: Option<String>,
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
#[serde(rename_all = "camelCase")]
pub enum NftScarcity {
    Unique,
    Legendary,
    Epic,
    Rare,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintNft {
    pub scarcity: NftScarcity,
    pub ticker: Option<String>,
    pub fees: Fees,
    pub rate: u32,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "function")]
pub enum Action {
    CreateFee(CreateFee),

    Transfer(Transfer),

    Configure(Configure),

    Evolve(Evolve),

    MintNft(MintNft),

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
