use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PackScarcity {
    Legendary([String; 2]),
    Epic([String; 3]),
    Rare([String; 4]),
}

impl Default for PackScarcity {
    fn default() -> Self {
        Self::Legendary(["1".to_string(), "2".to_string()])
    }
}

impl From<&PackScarcity> for Vec<String> {
    fn from(item: &PackScarcity) -> Self {
        match item {
            PackScarcity::Legendary(nfts) => nfts.to_vec(),
            PackScarcity::Epic(nfts) => nfts.to_vec(),
            PackScarcity::Rare(nfts) => nfts.to_vec(),
        }
    }
}

impl<'a> IntoIterator for &'a PackScarcity {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(self).into_iter()
    }
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Pack {
    pub id: String,
    pub nfts: PackScarcity,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub super_operator: String,
    pub operators: Vec<String>,

    /// Address of the attached ERC1155-compliant contract
    pub erc1155: String,

    /// NOTE: Currently only Pianity is allowed to do mints and transfers which means that
    /// ownership always defaults to Pianity. This field represents the address to which ownership
    /// always defaults in the ERC1155 contract.
    ///
    /// It is required in order to, for example, determine whether a transfer represents a sell or
    /// a resell.
    pub custodian: String,

    /// Token ID of the token used for paying
    pub exchange_token: String,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub name: Option<String>,

    pub settings: Settings,

    pub packs: HashMap<String, Pack>,

    pub evolve: Option<String>,
    pub can_evolve: Option<bool>,
}
