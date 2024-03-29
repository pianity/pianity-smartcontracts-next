use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use kv_storage::{kv, KvStorage};

use crate::contract_utils::js_imports::Kv;

/**
 * address -> share
 */
pub type Royalties = HashMap<String, u32>;

#[derive(Debug)]
#[kv(impl = "Kv")]
pub struct AttachedRoyalties {
    pub base_id: String,
    pub royalties: Royalties,
    pub rate: u32,
    // NOTE: The following will only be necessary when artists will be allowed to mint instead of
    // Pianity. At the moment Pianity is acting as the custodian so we can just check for the
    // `custodian` field in the contract's settings.
    //
    // /// Address of the original owner of the NFT
    // ///
    // /// NOTE: This is required because in order to determine whether a transfer is a sell or a
    // /// resell, we have to know who originally owned the NFT.
    // ///
    // /// In theory this should be obtained by simply checking what's the address that minted the NFT
    // /// however it isn't possible without using the unsafe client so we resort to manually storing
    // /// the address.
    // pub minter: String,
}

#[kv(impl = "Kv", subpath)]
pub struct Settings {
    pub paused: bool,

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
}

// #[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[kv(impl = "Kv")]
pub struct State {
    #[kv(subpath)]
    settings: Settings,
    #[kv(map)]
    all_attached_royalties: AttachedRoyalties,
}
