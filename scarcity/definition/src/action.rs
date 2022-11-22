use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

use crate::error::ContractError;
use crate::state::{Royalties, State};

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRoyalties {
    pub base_id: String,
    pub royalties: Royalties,
    pub rate: u32,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditAttachedRoyalties {
    pub base_id: String,
    pub royalties: Royalties,
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

impl From<&Scarcity> for u32 {
    fn from(scarcity: &Scarcity) -> Self {
        match scarcity {
            Scarcity::Unique => 1,
            Scarcity::Legendary => 10,
            Scarcity::Epic => 100,
            Scarcity::Rare => 1000,
        }
    }
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MintNft {
    pub scarcity: Scarcity,
    pub base_id: Option<String>,
    pub royalties: Royalties,
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

#[derive(JsonSchema, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Configure {
    pub paused: Option<bool>,
    pub super_operators: Option<Vec<String>>,
    pub operators: Option<Vec<String>>,
    pub erc1155: Option<String>,
    pub custodian: Option<String>,
    pub can_evolve: Option<bool>,
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
    EditAttachedRoyalties(EditAttachedRoyalties),
    AttachRoyalties(AttachRoyalties),
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
