use num_bigint::BigUint;
use num_traits::ToPrimitive;
use sha3::{Digest, Sha3_256};
use warp_shuffle::state::{ShuffleScarcity, State};

use crate::contract_utils::js_imports::log;

fn index_to_editions_count(n: usize) -> u32 {
    (0..n).fold(1, |acc, _| acc * 10)
}

pub fn get_all_nfts_ids(nfts: &ShuffleScarcity) -> Vec<String> {
    Vec::from(nfts)
        .iter()
        .enumerate()
        .flat_map(|(i, id)| {
            (0..index_to_editions_count(i)).map(move |edition| format!("{}-{}", edition + 1, id))
        })
        .collect()
}

pub fn is_op(state: &State, address: &str) -> bool {
    is_super_op(state, address) || state.settings.operators.contains(&address.into())
}

pub fn is_super_op(state: &State, address: &str) -> bool {
    state.settings.super_operators.contains(&address.into())
}

pub struct Rng {
    seed: String,
    inc: u32,
}

impl Rng {
    pub fn new(seed: String) -> Self {
        Self { seed, inc: 0 }
    }

    pub fn next(&mut self) -> f32 {
        self.inc += 1;

        let hash = Sha3_256::digest(&format!("{}{}", self.seed, self.inc));

        let bigint: u128 = BigUint::from_bytes_be(&hash)
            .modpow(&BigUint::from(1u8), &BigUint::from(f32::MAX as u128))
            .to_u128()
            .unwrap();

        bigint as f32 / f32::MAX
    }
}

// /// Convert
// fn scarcity_id_to_u32(scarcity: &str) -> Option<u32> {
//     match scarcity {
//         "UNIQUE" => 1,
//         "LEGENDARY" => 10,
//         "EPIC" => 100,
//         "RARE" => 1000,
//         _ => 0,
//     }
// }

#[derive(Debug)]
pub enum Scarcity {
    Unique,
    Legendary,
    Epic,
    Rare,
}

impl TryFrom<&str> for Scarcity {
    type Error = ();

    fn try_from(scarcity_raw: &str) -> Result<Self, Self::Error> {
        match scarcity_raw.to_lowercase().as_str() {
            "unique" => Ok(Self::Unique),
            "legendary" => Ok(Self::Legendary),
            "epic" => Ok(Self::Epic),
            "rare" => Ok(Self::Rare),
            _ => Err(()),
        }
    }
}

// impl Into<u32> for Scarcity {
//     fn into(self) -> u32 {
//         match self {
//             Self::Unique => 1,
//             Self::Legendary => 10,
//             Self::Epic => 100,
//             Self::Rare => 1000,
//         }
//     }
// }

impl From<&Scarcity> for u32 {
    fn from(scarcity: &Scarcity) -> Self {
        match scarcity {
            Scarcity::Unique => 1,
            Scarcity::Legendary => 10,
            Scarcity::Epic => 100,
            Scarcity::Rare => 1000,
        }
    }
}

impl ToString for Scarcity {
    fn to_string(&self) -> String {
        match self {
            Scarcity::Unique => "UNIQUE".to_string(),
            Scarcity::Legendary => "LEGENDARY".to_string(),
            Scarcity::Epic => "EPIC".to_string(),
            Scarcity::Rare => "RARE".to_string(),
        }
    }
}

pub struct NftBaseId {
    pub id: String,
    pub scarcity: Scarcity,
}

impl ToString for NftBaseId {
    fn to_string(&self) -> String {
        format!("{}-{}", self.scarcity.to_string(), self.id)
    }
}

pub struct NftId {
    pub id: String,
    pub scarcity: Scarcity,
    pub edition: u32,
}

impl TryFrom<&str> for NftId {
    type Error = ();

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        let splited = {
            let mut splited = id.splitn(3, '-');

            (
                splited.next().ok_or(())?,
                splited.next().ok_or(())?,
                splited.next().ok_or(())?,
            )
        };

        let scarcity = Scarcity::try_from(splited.1)?;
        let edition = splited.0.parse::<u32>().unwrap_or(0);

        if edition > u32::from(&scarcity) {
            Err(())
        } else {
            Ok(NftId {
                id: splited.2.to_string(),
                scarcity,
                edition,
            })
        }
    }
}

impl ToString for NftId {
    fn to_string(&self) -> String {
        format!("{}-{}-{}", self.edition, self.scarcity.to_string(), self.id)
    }
}

// fn is_prefix_valid(edition: &str, scarcity: &str) -> bool {
//     let edition = edition.parse::<u32>().unwrap_or(0);
//
//     let max_edition = match scarcity {
//         "UNIQUE" => 1,
//         "LEGENDARY" => 10,
//         "EPIC" => 100,
//         "RARE" => 1000,
//         _ => 0,
//     };
//
//     edition > 0 && edition <= max_edition
// }

/// 1-UNIQUE-TX_ID
pub fn splited_nft_id(id: &str) -> Option<NftId> {
    let splited = {
        let mut splited = id.splitn(3, '-');

        log(&format!("splited before: {:?}", splited));

        (splited.next()?, splited.next()?, splited.next()?)
    };

    log(&format!("splited: {:?}", splited));

    let scarcity = Scarcity::try_from(splited.1).ok()?;

    log(&format!("scarcity: {:?}", scarcity));

    let edition = splited.0.parse::<u32>().unwrap_or(0);

    log(&format!("edition: {}", edition));

    if edition > u32::from(&scarcity) {
        log(&format!("more than: {}", u32::from(&scarcity)));
        None
    } else {
        Some(NftId {
            id: splited.2.to_string(),
            scarcity,
            edition,
        })
    }
}
