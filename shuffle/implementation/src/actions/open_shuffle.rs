use std::{cmp::Ordering, collections::HashMap};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, BalancePrecision, State as Erc1155State, Token as Erc1155Token},
};

use warp_shuffle::{
    action::{ActionResult, BoostOpenShuffle, HandlerResult, OpenShuffle},
    error::ContractError,
    state::{ShuffleScarcity, State},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::write_foreign_contract,
    utils::{splited_nft_id, NftId, Rng},
};
use crate::{
    contract_utils::{
        foreign_call::read_foreign_contract_state,
        js_imports::{log, Vrf},
    },
    utils::get_all_nfts_ids,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

fn is_nft_available(
    nft_id: &String,
    tokens: &HashMap<String, Erc1155Token>,
    custodian: &String,
) -> bool {
    tokens
        .get(nft_id)
        .unwrap() // NFT existence is checked in `mint_shuffle` action
        .balances
        .iter()
        .next()
        .unwrap() // NFT having only one balance is checked in `mint_shuffle` action
        .0
        == custodian
}

fn get_available_nfts(
    nfts: &ShuffleScarcity,
    tokens: &HashMap<String, Erc1155Token>,
    custodian: &String,
) -> Vec<Vec<String>> {
    log(&format!("getting all nfts ids"));
    let all_nfts = get_all_nfts_ids(nfts);
    log(&format!("all nfts ids: {:?}", all_nfts));

    let all_available_nfts = all_nfts
        .into_iter()
        .filter(|nft_id| is_nft_available(nft_id, tokens, custodian))
        .collect::<Vec<_>>();

    log(&format!("all available nfts: {:?}", all_available_nfts));

    log(&format!(
        "grouping nfts by rarity, nfts len: {}",
        nfts.into_iter().count() - 1
    ));

    let nfts_per_scarcity = (0..nfts.into_iter().count())
        .map(|i| {
            log(&format!(
                "getting nfts for rarity {}, {}",
                i,
                nfts.into_iter().count()
            ));

            all_available_nfts
                .clone()
                .into_iter()
                .filter(|id| {
                    log(&format!("filtering {}", id));

                    let edition = splited_nft_id(id).unwrap().edition;

                    log(&format!(
                        "edition {}: {}; {}, {}",
                        id,
                        edition,
                        i,
                        10u32.pow(i as u32)
                    ));

                    let test = edition >= 1 && edition <= 10u32.pow(i as u32);

                    log(&format!("filtering condition: {}", test));

                    test
                })
                .collect()
        })
        .collect::<Vec<Vec<String>>>();

    log(&format!("grouped nfts by rarity: {:?}", nfts_per_scarcity));

    return nfts_per_scarcity;
}

fn draw_nft(nfts: Vec<Vec<String>>, boost: f32) -> Option<String> {
    let scarcity = nfts.len() as f32;

    log("drawing");

    let mut rng = Rng::new(Vrf::value());

    let drawn_scarcity = {
        let drawn_scarcity = nfts
            .iter()
            .zip(nfts.iter().enumerate().map(|(i, _)| {
                let s = 10f32.powi(i as i32);
                s / scarcity + (s / scarcity) * boost
            }))
            .find(|(nfts, luck)| nfts.len() > 0 && rng.next() < *luck)
            .map(|(nfts, _)| nfts);

        if drawn_scarcity.is_none() {
            log(&format!("drawn scarcity is none!"));
            let find = nfts.iter().rev().find(|nfts| nfts.len() > 0);

            log(&format!("find: {:?}; {:?}", find, nfts));

            find
        } else {
            drawn_scarcity
        }
    };

    if drawn_scarcity.is_none() {
        log(&format!("drawn none!"));
        None
    } else {
        let drawn_scarcity = drawn_scarcity.unwrap();
        let r = rng.next();
        let i = (r * (drawn_scarcity.len() - 1) as f32).round() as usize;
        log(&format!(
            "drawn r: {}, i: {}, len: {}",
            r,
            i,
            drawn_scarcity.len()
        ));
        Some(drawn_scarcity[i].clone())
    }

    // let i = (rng.next() * drawn_scarcity.len() as f32).round() as usize;
    // log(&format!("drawn {}, {}", i, drawn_scarcity.len()));

    // drawn_scarcity.map(|drawn_scarcity| drawn_scarcity[i].clone())
}

#[async_trait(?Send)]
impl AsyncActionable for OpenShuffle {
    async fn action(self, caller: String, state: State) -> ActionResult {
        let owner = self.owner.unwrap_or_else(|| caller.clone());

        let (boost, boost_price) = {
            let BoostOpenShuffle {
                boost,
                shuffle_price,
            } = self.boost.unwrap_or_default();

            if boost < 0. || boost > 1. || boost > state.settings.boost_cap {
                return Err(ContractError::BoostCapExceeded);
            }

            Ok((
                boost,
                Balance::new(
                    (state.settings.boost_price_modifier * boost * shuffle_price.value as f32)
                        as BalancePrecision,
                ),
            ))
        }?;

        let shuffle = state
            .shuffles
            .get(&self.shuffle_id)
            .ok_or_else(|| ContractError::ShuffleNotFound(self.shuffle_id.clone()))?;

        log(&format!("get erc1155 state"));

        let erc1155_state = read_foreign_contract_state::<Erc1155State>(&state.settings.erc1155)
            .await
            .map_err(|_err| ContractError::Erc1155ReadFailed)?;

        log(&format!("got erc1155 state"));

        let owner_balance = erc1155_state
            .tokens
            .get(&self.shuffle_id)
            .ok_or(ContractError::TokenNotFound(self.shuffle_id.clone()))?
            .balances
            .get(&owner)
            .map_or(0, |balance| balance.value);

        if owner_balance < 1 {
            // TODO: Refactor this error to include useful information or no information at all
            return Err(ContractError::CallerBalanceNotEnough(0));
        }

        let nfts = get_available_nfts(
            &shuffle.nfts,
            &erc1155_state.tokens,
            &state.settings.custodian,
        );

        let nft =
            draw_nft(nfts, boost).ok_or(ContractError::NoNftAvailable(self.shuffle_id.clone()))?;

        // let nft_i = match nfts.len() {
        //     0 => return Err(ContractError::NoNftAvailable(self.shuffle_id.clone())),
        //     1 => 0,
        //     len => Vrf::random_int((len - 1) as i32),
        // };
        // let nft = nfts.get(nft_i as usize).unwrap();

        log(&format!("sending tokens"));

        let mut batch = vec![
            Erc1155Action::Action::Burn(Erc1155Action::Burn {
                token_id: self.shuffle_id,
                owner: Some(owner.clone()),
                qty: Balance::new(1),
            }),
            Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
                token_id: nft.to_string(),
                from: None,
                to: owner.clone(),
                qty: Balance::new(1),
            }),
        ];

        if boost_price.value > 0 {
            batch.push(Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
                token_id: state.settings.boost_token.clone(),
                from: Some(owner),
                to: state.settings.custodian.clone(),
                qty: boost_price,
            }));
        }

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: batch }),
        )
        .await
        .map_err(|err| ContractError::Erc1155Error(err))?;

        log(&format!("byebye"));

        // let transfer = ;
        //
        // write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
        //     &state.settings.erc1155,
        //     transfer,
        // )
        // .await
        // .map_err(|err| ContractError::Erc1155Error(err))?;

        Ok(HandlerResult::Write(state))
    }
}
