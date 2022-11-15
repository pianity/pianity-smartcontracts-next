use warp_scarcity::{action::Scarcity, state::State};

use crate::contract_utils::js_imports::log;

pub fn is_op(state: &State, address: &str) -> bool {
    is_super_op(state, address) || state.settings.operators.contains(&address.into())
}

pub fn is_super_op(state: &State, address: &str) -> bool {
    state.settings.super_operators.contains(&address.into())
}

fn is_nft_prefix_valid(edition: &str, scarcity: &str) -> bool {
    let edition = edition.parse::<u32>().unwrap_or(0);

    let max_edition = match scarcity {
        "UNIQUE" => 1,
        "LEGENDARY" => 10,
        "EPIC" => 100,
        "RARE" => 1000,
        _ => 0,
    };

    edition > 0 && edition <= max_edition
}

// TODO: This code is mostly duplicated from the Shuffle contract. It should be refactored to be
// shared instead.
pub struct NftId {
    pub edition: u32,
    pub scarcity: Scarcity,
    pub base_id: String,
}

impl ToString for NftId {
    fn to_string(&self) -> String {
        format!(
            "{}-{}-{}",
            self.edition,
            self.scarcity.to_string(),
            self.base_id
        )
    }
}

pub struct ShuffleId {
    pub base_id: String,
}

impl ToString for ShuffleId {
    fn to_string(&self) -> String {
        format!("SHUFFLE-{}", self.base_id.clone())
    }
}

pub enum TokenId {
    Nft(NftId),
    Shuffle(ShuffleId),
    Token(String),
}

pub fn parse_token_id(id: &str) -> TokenId {
    let shuffle_split = id.splitn(2, "-").collect::<Vec<&str>>();
    let nft_split = id.splitn(3, "-").collect::<Vec<&str>>();

    if shuffle_split.len() == 2 && shuffle_split[0] == "SHUFFLE" {
        TokenId::Shuffle(ShuffleId {
            base_id: shuffle_split[1].to_string(),
        })
    } else if nft_split.len() == 3 && is_nft_prefix_valid(nft_split[0], nft_split[1]) {
        TokenId::Nft(NftId {
            edition: nft_split[0].parse().unwrap(),
            scarcity: Scarcity::try_from(nft_split[1]).unwrap(),
            base_id: nft_split[2].to_string(),
        })
    } else {
        TokenId::Token(id.to_string())
    }
}
