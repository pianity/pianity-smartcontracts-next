use serde::{Deserialize, Serialize};

use crate::contract_utils::js_imports::Kv;
use kv_storage::{kv, KvStorage};

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

#[derive(Serialize, Deserialize, Copy, Clone, Default, Debug, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct Balance {
    #[serde(with = "string")]
    pub value: BalancePrecision,
}

impl Balance {
    pub fn new(value: BalancePrecision) -> Self {
        Self { value }
    }
}

#[kv(impl = "Kv", subpath)]
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Approvals {
    #[kv(map)]
    pub approves: bool,
}

#[kv(impl = "Kv", subpath)]
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Token {
    pub ticker: String,
    pub tx_id: Option<String>,
    #[kv(map)]
    pub balances: Balance,
}

#[kv(impl = "Kv", subpath)]
pub struct Settings {
    pub default_token: String,

    pub paused: bool,

    pub super_operators: Vec<String>,
    pub operators: Vec<String>,

    pub proxies: Vec<String>,

    pub allow_free_transfer: bool,
}

#[kv(impl = "Kv")]
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct State {
    pub ticker_nonce: u32,
    #[kv(map, subpath)]
    pub tokens: Token,
    #[kv(map, subpath)]
    pub approvals: Approvals,
    #[kv(subpath)]
    pub settings: Settings,
}
