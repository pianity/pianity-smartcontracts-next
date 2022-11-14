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
    action::{ActionResult, BoostOpenShuffle, HandlerResult, OpenShuffleBatch},
    error::ContractError,
    state::{ShuffleBaseIds, State},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::write_foreign_contract,
    utils::{splited_nft_id, NftId, Rng, Scarcity},
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
    nfts: &ShuffleBaseIds,
    tokens: &HashMap<String, Erc1155Token>,
    custodian: &String,
) -> Option<Vec<Vec<String>>> {
    let all_nfts = get_all_nfts_ids(nfts);

    let all_available_nfts = all_nfts
        .into_iter()
        .filter(|nft_id| is_nft_available(nft_id, tokens, custodian))
        .collect::<Vec<_>>();

    let nfts_per_scarcity = (0..nfts.into_iter().count())
        .map(|i| {
            all_available_nfts
                .clone()
                .into_iter()
                .filter(|id| {
                    let edition = splited_nft_id(id).unwrap().edition;
                    edition >= 1 && edition <= 10u32.pow(i as u32)
                })
                .collect::<Vec<String>>()
        })
        .filter(|editions| !editions.is_empty())
        .collect::<Vec<Vec<String>>>();

    (!nfts_per_scarcity.is_empty()).then_some(nfts_per_scarcity)
}

fn draw_nft(shuffle: &Vec<Vec<String>>, boost: f32) -> &String {
    let mut rng = Rng::new(Vrf::value());

    let drawn_scarcity = {
        let editions_count = shuffle
            .iter()
            .map(|editions| editions.len() as f32)
            .sum::<f32>();

        let raw_odds =
            shuffle
                .iter()
                .enumerate()
                .fold(Vec::new(), |mut odds, (scarcity_index, editions)| {
                    let boost = if scarcity_index == shuffle.len() {
                        1.
                    } else {
                        1. + boost
                    };

                    odds.push(
                        odds.last().unwrap_or(&0.)
                            + (editions.len() as f32 / editions_count) * boost,
                    );

                    odds
                });

        let odds = raw_odds.iter().map(|odd| odd / raw_odds.last().unwrap());

        let r_scarcity = rng.next();

        shuffle
            .iter()
            .zip(odds)
            .find(|(_, odd)| r_scarcity < *odd)
            .map(|(editions, _)| editions)
            .unwrap()
    };

    let drawn_nft = {
        let r_nft = rng.next();

        let drawn_nft_index = (r_nft * (drawn_scarcity.len() as f32 - 1.).round()) as usize;

        drawn_scarcity.get(drawn_nft_index).unwrap()
    };

    drawn_nft
}

#[async_trait(?Send)]
impl AsyncActionable for OpenShuffleBatch {
    async fn action(self, caller: String, state: State) -> ActionResult {
        let owner = self.owner.unwrap_or_else(|| caller.clone());

        let (boost, boost_price) = {
            let BoostOpenShuffle {
                boost,
                shuffle_price,
            } = self.boost.unwrap_or_default();

            if !(0. ..=1.).contains(&boost) || boost > state.settings.boost_cap {
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

        let erc1155_state = read_foreign_contract_state::<Erc1155State>(&state.settings.erc1155)
            .await
            .map_err(|_err| ContractError::Erc1155ReadFailed)?;

        let owner_balance = erc1155_state
            .tokens
            .get(&self.shuffle_id)
            .ok_or_else(|| ContractError::TokenNotFound(self.shuffle_id.clone()))?
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
        )
        .ok_or_else(|| ContractError::NoNftAvailable(self.shuffle_id.clone()))?;

        let nft = draw_nft(&nfts, boost);

        let mut batch = vec![
            Erc1155Action::Action::Burn(Erc1155Action::Burn {
                token_id: self.shuffle_id,
                owner: Some(owner.clone()),
                qty: Balance::new(1),
            }),
            // TODO: Interact with the Scarcity contract to pay for the NFT. As it is now
            // (interacting directly with the ERC1155 contract), the royalties are bypassed.
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
        .map_err(ContractError::Erc1155Error)?;

        Ok(HandlerResult::Write(state))
    }
}
