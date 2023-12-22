use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use warp_erc1155::state::Balance;

// TODO: Find a way to export `UNIT` via schemars or put it in `Settings`.
/// The exact amount that all the sum of all the fees of a token must be equal to.
pub const UNIT: u32 = 1_000_000;

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub paused: bool,

    pub super_operators: Vec<String>,
    pub operators: Vec<String>,

    /// Address of the attached ERC1155-compliant contract
    pub erc1155: String,
    // /// TODO: - allow to have multiple lockable tokens
    // ///       - should it be set in the settings and restricted on these tokens or should operators
    // ///       have to specify what tokens are lockable?
    // ///       - in case of the latter, it should be made configurable
    // ///
    // /// Token ID of the token used for paying
    // pub exchange_token: String,
}

#[derive(JsonSchema, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cliff {
    pub token_id: String,
    pub from: String,
    pub qty: Balance,
    pub at: u32,
    pub duration: u32,
}

#[derive(JsonSchema, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Linear {
    pub token_id: String,
    pub from: String,
    pub qty: Balance,
    pub at: u32,
    pub duration: u32,
    pub unlocked: Balance,
}

#[derive(JsonSchema, Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LockedBalance {
    Cliff(Cliff),
    Linear(Linear),
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
pub struct InitialState {
    pub settings: Settings,
    pub vault: HashMap<String, Vec<LockedBalance>>,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    pub name: String,

    pub initial_state: Option<InitialState>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolve: Option<String>,
    pub can_evolve: bool,
}
