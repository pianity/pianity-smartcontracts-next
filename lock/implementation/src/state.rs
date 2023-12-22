use serde::{Deserialize, Serialize};

use kv_storage::{kv, KvStorage};

use warp_lock::state::LockedBalance;

use crate::contract_utils::js_imports::Kv;

// TODO: Find a way to export `UNIT` via schemars or put it in `Settings`.
/// The exact amount that all the sum of all the fees of a token must be equal to.
pub const UNIT: u32 = 1_000_000;

#[kv(impl = "Kv", subpath)]
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

#[kv(impl = "Kv")]
pub struct State {
    #[kv(subpath)]
    pub settings: Settings,
    #[kv(map)]
    pub vault: Vec<LockedBalance>,
}
