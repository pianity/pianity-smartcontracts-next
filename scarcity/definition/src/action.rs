use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

use crate::error::ContractError;
use crate::state::{Fees, State};

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachFee {
    pub base_id: String,
    pub fees: Fees,
    pub rate: u32,
}

// TODO: This code is mostly duplicated from the Shuffle contract. It should be refactored to be
// shared instead.
#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scarcity {
    Unique,
    Legendary,
    Epic,
    Rare,
}

impl TryFrom<&str> for Scarcity {
    type Error = ();

    fn try_from(scarcity_raw: &str) -> Result<Self, Self::Error> {
        match scarcity_raw.to_lowercase().as_str() {
            "unique" => Ok(Self::Unique),
            "legendary" => Ok(Self::Legendary),
            "epic" => Ok(Self::Epic),
            "rare" => Ok(Self::Rare),
            _ => Err(()),
        }
    }
}

impl ToString for Scarcity {
    fn to_string(&self) -> String {
        match self {
            Scarcity::Unique => "UNIQUE".to_string(),
            Scarcity::Legendary => "LEGENDARY".to_string(),
            Scarcity::Epic => "EPIC".to_string(),
            Scarcity::Rare => "RARE".to_string(),
        }
    }
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintNft {
    pub scarcity: Scarcity,
    pub ticker: Option<String>,
    pub fees: Fees,
    pub rate: u32,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub token_id: String,
    pub price: Balance,
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
    AttachFee(AttachFee),

    MintNft(MintNft),

    Transfer(Transfer),

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
