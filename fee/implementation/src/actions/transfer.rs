use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State},
};

use warp_fee::{
    action::{ActionResult, CreateFee, HandlerResult, Transfer},
    error::ContractError,
    state::{Fees, Nft, State, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    actions::{Actionable, AsyncActionable},
    contract_utils::foreign_call::write_foreign_contract,
};
use crate::{
    contract_utils::{foreign_call::read_foreign_contract_state, js_imports::log},
    utils::splited_nft_id,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

async fn get_token_owner(erc1155: &str, nft_id: &str) -> Option<String> {
    let state = read_foreign_contract_state::<Erc1155State>(&erc1155.to_string()).await;

    let token = state.tokens.get(nft_id)?;

    let owner = token.balances.iter().next().unwrap_throw();

    Some(owner.0.clone())
}

#[async_trait(?Send)]
impl AsyncActionable for Transfer {
    async fn action(self, _caller: String, state: State) -> ActionResult {
        let nft_base_id = splited_nft_id(&self.nft_id)
            .ok_or_else(|| ContractError::InvalidNftId(self.nft_id.clone()))?
            .2;

        let nft = state
            .nfts
            .get(nft_base_id)
            .ok_or_else(|| ContractError::TokenNotFound(self.nft_id.clone()))?;

        let nft_owner = get_token_owner(&state.settings.erc1155, &self.nft_id)
            .await
            .ok_or(ContractError::TokenOwnerNotFound)?;

        let is_resell = nft_owner != state.settings.custodian;

        let rate = if is_resell { nft.rate } else { UNIT };

        let mut transfers: Vec<Erc1155Action::Transfer> = Vec::new();

        if self.price.value > 0 {
            // If this transfer is a resell, pay the NFT owner.
            if is_resell {
                transfers.push(Erc1155Action::Transfer {
                    from: Some(self.to.clone()),
                    to: nft_owner.clone(),
                    token_id: state.settings.exchange_token.clone(),
                    qty: Balance::new(self.price.value * (rate / UNIT)),
                });
            }

            // Pay the share holders.
            nft.fees.iter().for_each(|(address, share)| {
                let fee_amount = (self.price.value as f32
                    * (*share as f32 / (UNIT as f32 * (UNIT as f32 / rate as f32))))
                    as u32;

                transfers.push(Erc1155Action::Transfer {
                    from: Some(self.to.clone()),
                    to: address.clone(),
                    token_id: state.settings.exchange_token.clone(),
                    qty: Balance::new(fee_amount),
                });
            });
        }

        // Transfer the NFT.
        transfers.push(Erc1155Action::Transfer {
            from: Some(nft_owner),
            to: self.to.clone(),
            token_id: self.nft_id,
            qty: Balance::new(1),
        });

        let transfers = transfers
            .into_iter()
            .map(|transfer| Erc1155Action::Action::Transfer(transfer))
            .collect();

        let transaction_batch =
            Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: transfers });

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            transaction_batch,
        )
        .await
        .or_else(|err| Err(ContractError::Erc1155Error(err)))?;

        Ok(HandlerResult::Write(state))
    }
}