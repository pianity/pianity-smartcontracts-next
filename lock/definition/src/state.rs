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
    pub can_evolve: bool,

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

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
pub struct LockedBalance {
    pub token_id: String,
    pub from: String,
    pub qty: Balance,
    pub at: u32,
    pub duration: u32,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub name: String,

    pub settings: Settings,

    pub vault: HashMap<String, Vec<LockedBalance>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolve: Option<String>,
}
