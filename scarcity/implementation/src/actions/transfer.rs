use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, BalancePrecision, State as Erc1155State},
};

use warp_scarcity::{
    action::{ActionResult, AttachRoyalties, HandlerResult, Transfer},
    error::ContractError,
    state::{AttachedRoyalties, Royalties, State, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    actions::AsyncActionable,
    contract_utils::{
        foreign_call::{ForeignContractCaller, ForeignContractState},
        js_imports::log,
    },
    utils::{NftId, ShuffleId, TokenId},
};

async fn get_token_owner(
    foreign_caller: &mut ForeignContractCaller,
    erc1155: &str,
    nft_id: &str,
) -> Result<String, ContractError> {
    let state = match foreign_caller
        .read(&erc1155.to_string())
        .await
        .map_err(|_err| ContractError::Erc1155ReadFailed)?
    {
        ForeignContractState::Erc1155(state) => state,
    };

    let token = state
        .tokens
        .get(nft_id)
        .ok_or(ContractError::TokenOwnerNotFound)?;

    let owner = token.balances.iter().next().unwrap_throw();

    Ok(owner.0.clone())
}

#[async_trait(?Send)]
impl AsyncActionable for Transfer {
    async fn action(
        self,
        _caller: String,
        state: State,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let base_id = match TokenId::from(self.token_id.as_ref()) {
            TokenId::Nft(NftId { base_id, .. }) => Ok(base_id),
            TokenId::Shuffle(ShuffleId { base_id }) => Ok(base_id),
            _ => Err(ContractError::InvalidTokenId),
        }?;

        let attached_royalties = state
            .all_attached_royalties
            .get(&base_id)
            // TODO: Rename TokenNotFound to NoRoyaltiesAttached
            .ok_or_else(|| ContractError::TokenNotFound(self.token_id.clone()))?;

        // let nft_owner =
        //     get_token_owner(foreign_caller, &state.settings.erc1155, &self.token_id).await?;

        let token_owner = self.from.clone();

        let is_resell = self.from != state.settings.custodian;

        let rate = if is_resell {
            attached_royalties.rate
        } else {
            UNIT
        };

        let mut transfers: Vec<Erc1155Action::Transfer> = Vec::new();

        if self.price.value > 0 {
            // If this transfer is a resell, pay the NFT owner.
            if is_resell {
                transfers.push(Erc1155Action::Transfer {
                    from: Some(self.to.clone()),
                    to: token_owner.clone(),
                    token_id: None,
                    qty: Balance::new(
                        (self.price.value as f32 * (rate as f32 / UNIT as f32)) as BalancePrecision,
                    ),
                });
            }

            // Pay the share holders.
            attached_royalties
                .royalties
                .iter()
                .for_each(|(address, royalty_share)| {
                    let royalty_amount = (self.price.value as f32
                        * (*royalty_share as f32 / (UNIT as f32 * (UNIT as f32 / rate as f32))))
                        as BalancePrecision;

                    transfers.push(Erc1155Action::Transfer {
                        from: Some(self.to.clone()),
                        to: address.clone(),
                        token_id: None,
                        qty: Balance::new(royalty_amount),
                    });
                });
        }

        // Transfer the token.
        transfers.push(Erc1155Action::Transfer {
            from: Some(token_owner),
            to: self.to.clone(),
            token_id: Some(self.token_id),
            qty: Balance::new(1),
        });

        let transfers = transfers
            .into_iter()
            .map(|transfer| Erc1155Action::Action::Transfer(transfer))
            .collect();

        let transaction_batch =
            Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: transfers });

        foreign_caller
            .write::<Erc1155ContractError, Erc1155Action::Action>(
                &state.settings.erc1155,
                transaction_batch,
            )
            .await
            .or_else(|err| Err(ContractError::Erc1155Error(err)))?;

        Ok(HandlerResult::Write(state))
    }
}
