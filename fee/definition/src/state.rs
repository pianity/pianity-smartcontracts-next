use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// TODO: Find a way to export `UNIT` via schemars or put it in `Settings`.
/// The exact amount that all the sum of all the fees of a token must be equal to.
pub const UNIT: u32 = 1_000_000;

/**
 * address -> share
 */
pub type Fees = HashMap<String, u32>;
// pub type Fees = Vec<(String, u32)>;

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    pub id: String,
    pub fees: Fees,
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

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub super_operator: String,
    pub operators: Vec<String>,

    // NOTE: Removed until a generalistic solution has been found.
    // /// Addesses authorized to interact with the contract.
    // ///
    // /// If empty, all addresses are authorized.
    // pub authorized_addresses: Vec<String>,
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

    pub nfts: HashMap<String, Nft>,

    pub evolve: Option<String>,
    pub can_evolve: Option<bool>,
}
