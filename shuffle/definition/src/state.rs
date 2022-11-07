use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

struct NftBaseId {
    id: String,
    scarcity: String,
}

struct NftId {
    base_id: NftBaseId,
    edition: String,
}

// TODO: Rename this to ShuffleBaseIds
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShuffleBaseIds {
    Legendary([String; 2]),
    Epic([String; 3]),
    Rare([String; 4]),
}

impl Default for ShuffleBaseIds {
    fn default() -> Self {
        Self::Legendary(["1".to_string(), "2".to_string()])
    }
}

impl From<&ShuffleBaseIds> for Vec<String> {
    fn from(item: &ShuffleBaseIds) -> Self {
        match item {
            ShuffleBaseIds::Legendary(nfts) => nfts.to_vec(),
            ShuffleBaseIds::Epic(nfts) => nfts.to_vec(),
            ShuffleBaseIds::Rare(nfts) => nfts.to_vec(),
        }
    }
}

impl<'a> IntoIterator for &'a ShuffleBaseIds {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        Vec::from(self).into_iter()
    }
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Shuffle {
    pub id: String,
    pub nfts: ShuffleBaseIds,
}

// pub struct BoostRules {
//     pub legendary: u32,
//     pub epic
// }

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub super_operators: Vec<String>,
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

    /// Id of the token used to boost shuffles luck
    pub boost_token: String,
    pub boost_price_modifier: f32,
    /// 0 <= boost_cap <= 1
    pub boost_cap: f32,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub name: Option<String>,

    pub settings: Settings,

    pub shuffles: HashMap<String, Shuffle>,

    pub evolve: Option<String>,
    pub can_evolve: Option<bool>,
}
