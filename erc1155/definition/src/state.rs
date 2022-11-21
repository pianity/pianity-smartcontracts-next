use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

pub type BalancePrecision = u64;

#[derive(JsonSchema, Serialize, Deserialize, Copy, Clone, Default, Debug, Hash, PartialEq, Eq)]
#[serde(rename_all = "camelCase", transparent)]
pub struct Balance {
    #[serde(with = "string")]
    #[schemars(with = "String")]
    pub value: BalancePrecision,
}

impl Balance {
    pub fn new(value: BalancePrecision) -> Self {
        Self { value }
    }
}

pub type Balances = HashMap<String, Balance>;
pub type Approvals = HashMap<String, bool>;

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub ticker: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<String>,
    pub balances: Balances,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub paused: bool,

    pub super_operators: Vec<String>,
    pub operators: Vec<String>,

    pub proxies: Vec<String>,

    pub allow_free_transfer: bool,
}

#[derive(JsonSchema, Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub name: String,

    pub settings: Settings,

    pub default_token: String,
    pub ticker_nonce: u64,
    pub tokens: HashMap<String, Token>,
    pub approvals: HashMap<String, Approvals>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evolve: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_evolve: Option<bool>,
}
