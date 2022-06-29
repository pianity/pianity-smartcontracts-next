use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Balances = HashMap<String, u64>;
pub type Approvals = HashMap<String, bool>;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub ticker: String,
    pub balances: Balances,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub super_owner: String,
    pub owners: Vec<String>,

    /// Addesses authorized to interact with the contract.
    ///
    /// If empty, all addresses are authorized.
    pub authorized_addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub name: Option<String>,

    pub settings: Settings,

    pub tokens: HashMap<String, Token>,
    pub approvals: HashMap<String, Approvals>,

    pub evolve: Option<String>,
    pub can_evolve: Option<bool>,
}
