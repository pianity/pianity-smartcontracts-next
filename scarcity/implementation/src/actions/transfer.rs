use async_trait::async_trait;

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, BalancePrecision},
};

use warp_scarcity::{
    action::{ActionResult, HandlerResult, Transfer},
    error::ContractError,
    state::{Parameters, UNIT},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::State,
    utils::{NftId, ShuffleId, TokenId},
};

#[async_trait(?Send)]
impl AsyncActionable for Transfer {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let qty = self.qty.unwrap_or(Balance::new(1));

        if qty.value < 1 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        let base_id = match TokenId::from(self.token_id.as_ref()) {
            TokenId::Nft(NftId { base_id, .. }) => {
                if qty.value != 1 {
                    Err(ContractError::QtyMustBeOneForNftTransfers)?
                } else {
                    Ok(base_id)
                }
            }
            TokenId::Shuffle(ShuffleId { base_id }) => Ok(base_id),
            TokenId::Token(token_id) => {
                Err(ContractError::CantUseTransferWithSimpleTokens(token_id))
            }
        }?;

        let attached_royalties = State::all_attached_royalties(&base_id)
            .ok_or(ContractError::RoyaltiesNotFound(self.token_id.clone()))
            .await?
            .get()
            .await;

        let token_owner = self.from.clone();

        let is_resell = self.from != State::settings().custodian().get().await;

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
                    from: Some(self.target.clone()),
                    target: token_owner.clone(),
                    token_id: None,
                    qty: Balance::new(
                        (self.price.value as f32 * ((UNIT - rate) as f32 / UNIT as f32))
                            as BalancePrecision,
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
                        from: Some(self.target.clone()),
                        target: address.clone(),
                        token_id: None,
                        qty: Balance::new(royalty_amount),
                    });
                });
        }

        // Transfer the token.
        transfers.push(Erc1155Action::Transfer {
            from: Some(token_owner),
            target: self.target.clone(),
            token_id: Some(self.token_id),
            qty,
        });

        let transfers = transfers
            .into_iter()
            .map(Erc1155Action::Action::Transfer)
            .collect();

        foreign_caller
            .write::<Erc1155ContractError, Erc1155Action::Action>(
                &State::settings().erc1155().get().await,
                Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: transfers }),
            )
            .await
            .map_err(ContractError::Erc1155Error)?;

        Ok(HandlerResult::Write(state))
    }
}
