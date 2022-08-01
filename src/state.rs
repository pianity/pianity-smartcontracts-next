use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

pub type BalancePrecision = u32;

#[derive(Serialize, Deserialize, Copy, Clone, Default, Debug, TS)]
#[serde(rename_all = "camelCase", transparent)]
#[ts(export)]
pub struct Balance {
    #[serde(with = "string")]
    pub value: BalancePrecision,
}

impl Balance {
    pub fn new(value: BalancePrecision) -> Self {
        Self { value }
    }
}

pub type Balances = HashMap<String, Balance>;
pub type Approvals = HashMap<String, bool>;

#[derive(Serialize, Deserialize, Clone, Default, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Token {
    pub ticker: String,
    pub balances: Balances,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Settings {
    pub super_operator: String,
    pub operators: Vec<String>,

    /// Addesses authorized to interact with the contract.
    ///
    /// If empty, all addresses are authorized.
    pub authorized_addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct State {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub settings: Settings,

    pub tokens: HashMap<String, Token>,
    pub approvals: HashMap<String, Approvals>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolve: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_evolve: Option<bool>,
}
