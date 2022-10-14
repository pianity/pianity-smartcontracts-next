use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State, Token as Erc1155Token},
};

use warp_packs::{
    action::{ActionResult, HandlerResult, OpenPack},
    error::ContractError,
    state::{PackScarcity, State},
};

use crate::{actions::AsyncActionable, contract_utils::foreign_call::write_foreign_contract};
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
        .unwrap() // NFT existence is checked in `mint_pack` action
        .balances
        .iter()
        .next()
        .unwrap() // NFT having only one balance is checked in `mint_pack` action
        .0
        == custodian
}

fn get_available_nfts(
    nfts: &PackScarcity,
    tokens: &HashMap<String, Erc1155Token>,
    custodian: &String,
) -> Vec<String> {
    get_all_nfts_ids(nfts)
        .into_iter()
        .filter(|nft_id| is_nft_available(nft_id, tokens, custodian))
        .collect()
}

#[async_trait(?Send)]
impl AsyncActionable for OpenPack {
    async fn action(self, caller: String, state: State) -> ActionResult {
        let owner = self.owner.unwrap_or_else(|| caller.clone());

        let pack = state
            .packs
            .get(&self.pack_id)
            .ok_or_else(|| ContractError::PackNotFound(self.pack_id.clone()))?;

        let erc1155_state = read_foreign_contract_state::<Erc1155State>(&state.settings.erc1155)
            .await
            .map_err(|_err| ContractError::Erc1155ReadFailed)?;

        let owner_balance = erc1155_state
            .tokens
            .get(&self.pack_id)
            .ok_or(ContractError::TokenNotFound(self.pack_id.clone()))?
            .balances
            .get(&owner)
            .map_or(0, |balance| balance.value);

        if owner_balance < 1 {
            // TODO: Refactor this error to include useful information or no information at all
            return Err(ContractError::CallerBalanceNotEnough(0));
        }

        let nfts: Vec<String> =
            get_available_nfts(&pack.nfts, &erc1155_state.tokens, &state.settings.custodian);

        let nft_i = match nfts.len() {
            0 => return Err(ContractError::NoNftAvailable(self.pack_id.clone())),
            1 => 0,
            len => Vrf::random_int((len - 1) as i32),
        };

        let nft = nfts.get(nft_i as usize).unwrap();

        let burn = Erc1155Action::Action::Burn(Erc1155Action::Burn {
            token_id: self.pack_id,
            owner: Some(owner.clone()),
            qty: Balance::new(1),
        });

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            burn,
        )
        .await
        .map_err(|err| ContractError::Erc1155Error(err))?;

        let transfer = Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
            token_id: nft.to_string(),
            from: None,
            to: owner,
            qty: Balance::new(1),
        });

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            transfer,
        )
        .await
        .map_err(|err| ContractError::Erc1155Error(err))?;

        Ok(HandlerResult::Write(state))
    }
}
