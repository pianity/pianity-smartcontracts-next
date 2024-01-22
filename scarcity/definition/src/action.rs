use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use warp_erc1155::{state::Balance};

use crate::error::ContractError;
use crate::state::{AttachedRoyalties, Parameters, Royalties};

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Initialize;

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRoyalties {
    pub base_id: String,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAllRoyalties;

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
    pub royalties: Option<Royalties>,
    pub rate: Option<u32>,
}

#[derive(JsonSchema, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveAttachedRoyalties {
    pub base_id: String,
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
        match scarcity_raw.to_uppercase().as_str() {
            "UNIQUE" => Ok(Self::Unique),
            "LEGENDARY" => Ok(Self::Legendary),
            "EPIC" => Ok(Self::Epic),
            "RARE" => Ok(Self::Rare),
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
    pub target: String,
    pub token_id: String,
    pub price: Balance,
    pub qty: Option<Balance>,
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
    Initialize(Initialize),
    GetRoyalties(GetRoyalties),
    GetAllRoyalties(GetAllRoyalties),
    AttachRoyalties(AttachRoyalties),
    EditAttachedRoyalties(EditAttachedRoyalties),
    RemoveAttachedRoyalties(RemoveAttachedRoyalties),
    MintNft(MintNft),
    Transfer(Transfer),
    Configure(Configure),
    Evolve(Evolve),
    Batch(Batch),
}

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ReadResponse {
    GetRoyalties((String, AttachedRoyalties)),
    GetAllRoyalties(Vec<(String, AttachedRoyalties)>),
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
