use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State},
};

use warp_shuffle::{
    action::{ActionResult, HandlerResult, MintShuffle},
    error::ContractError,
    state::{Shuffle, ShuffleBaseIds, State},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::{ForeignContractCaller, ForeignContractState},
};
use crate::{contract_utils::js_imports::Transaction, utils::get_all_nfts_ids};

// TODO: Do we actually want to check existence of NFTs to mint a shuffle?
async fn verify_nfts(
    erc1155: &String,
    nfts: &ShuffleBaseIds,
    all_shuffles: &HashMap<String, Shuffle>,
    foreign_caller: &mut ForeignContractCaller,
) -> Result<(), ContractError> {
    let nfts_vec: Vec<String> = nfts.into();

    for (shuffle_id, shuffle) in all_shuffles.iter() {
        for nft in &nfts_vec {
            if Vec::from(&shuffle.nfts).contains(nft) {
                return Err(ContractError::NftAlreadyInAShuffle(
                    shuffle_id.clone(),
                    nft.clone(),
                ));
            }
        }
    }

    let tokens = &match foreign_caller
        .read(erc1155)
        .await
        .map_err(|_err| ContractError::Erc1155ReadFailed)?
    {
        ForeignContractState::Erc1155(state) => state,
        _ => return Err(ContractError::Erc1155ReadFailed),
    }
    .tokens;

    for id in get_all_nfts_ids(nfts) {
        if !tokens.contains_key(&id) {
            return Err(ContractError::TokenNotFound(id));
        }
    }

    Ok(())
}

#[async_trait(?Send)]
impl AsyncActionable for MintShuffle {
    async fn action(
        self,
        _caller: String,
        mut state: State,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        verify_nfts(
            &state.settings.erc1155,
            &self.nfts,
            &state.shuffles,
            foreign_caller,
        )
        .await?;

        let total_editions = match &self.nfts {
            ShuffleBaseIds::Legendary(_) => 11,
            ShuffleBaseIds::Epic(_) => 111,
            ShuffleBaseIds::Rare(_) => 1111,
        };

        let prefix = "SHUFFLE";
        let shuffle_id = format!(
            "{}-{}",
            prefix,
            self.base_id.clone().unwrap_or_else(Transaction::id)
        );

        state.shuffles.insert(
            shuffle_id.clone(),
            Shuffle {
                id: shuffle_id,
                nfts: self.nfts,
            },
        );

        let erc1155_mint = Erc1155Action::Action::Mint(Erc1155Action::Mint {
            base_id: self.base_id,
            prefix: Some(prefix.to_string()),
            qty: Balance::new(total_editions),
        });

        foreign_caller
            .write::<Erc1155ContractError, Erc1155Action::Action>(
                &state.settings.erc1155,
                erc1155_mint,
            )
            .await
            .map_err(ContractError::Erc1155Error)?;

        Ok(HandlerResult::Write(state))
    }
}
