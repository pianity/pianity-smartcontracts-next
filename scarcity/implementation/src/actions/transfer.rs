use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, BalancePrecision, State as Erc1155State},
};

use warp_scarcity::{
    action::{ActionResult, CreateFee, HandlerResult, Transfer},
    error::ContractError,
    state::{Fees, Nft, State, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::{ForeignContractCaller, ForeignContractState},
    utils::splited_nft_id,
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
        let nft_base_id = splited_nft_id(&self.nft_id)
            .ok_or_else(|| ContractError::InvalidNftId(self.nft_id.clone()))?
            .2;

        let nft = state
            .nfts
            .get(nft_base_id)
            .ok_or_else(|| ContractError::TokenNotFound(self.nft_id.clone()))?;

        let nft_owner =
            get_token_owner(foreign_caller, &state.settings.erc1155, &self.nft_id).await?;

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
                    qty: Balance::new(self.price.value * (rate / UNIT) as BalancePrecision),
                });
            }

            // Pay the share holders.
            nft.fees.iter().for_each(|(address, share)| {
                let fee_amount = (self.price.value as f32 * (share / (UNIT * (UNIT / rate))) as f32)
                    as BalancePrecision;

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
