use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, AttachRoyalties, HandlerResult},
    error::ContractError,
    state::{Parameters, UNIT},
};

use crate::{
    actions::Actionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::{AttachedRoyalties, State},
};

use super::AsyncActionable;

pub async fn attach_royalties_internal(
    attach_royalties: &AttachRoyalties,
) -> Result<(), ContractError> {
    if attach_royalties.rate > UNIT {
        return Err(ContractError::InvalidRate);
    }

    // Check that the sum of all royalties is equal to UNIT
    let royalties_sum = attach_royalties
        .royalties
        .iter()
        .map(|(_, royalty)| *royalty)
        .reduce(|sum, royalty| sum + royalty)
        .unwrap_or(0);

    if royalties_sum != UNIT {
        return Err(ContractError::InvalidRoyalties);
    }

    // state.all_attached_royalties.insert(
    //     attach_royalties.base_id.clone(),
    //     AttachedRoyalties {
    //         base_id: attach_royalties.base_id.clone(),
    //         royalties: attach_royalties.royalties.clone(),
    //         rate: attach_royalties.rate,
    //     },
    // );

    State::all_attached_royalties(&attach_royalties.base_id)
        .set(&AttachedRoyalties {
            base_id: attach_royalties.base_id.clone(),
            royalties: attach_royalties.royalties.clone(),
            rate: attach_royalties.rate,
        })
        .await;

    Ok(())
}

#[async_trait(?Send)]
impl AsyncActionable for AttachRoyalties {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if State::all_attached_royalties(&self.base_id).exists().await {
            return Err(ContractError::TokenAlreadyExists(self.base_id));
        }

        // TODO: remove me
        // let erc1155 = match foreign_caller
        //     .read(&state.settings.erc1155.to_string())
        //     .await
        //     .map_err(|_err| ContractError::Erc1155ReadFailed)?
        // {
        //     ForeignContractState::Erc1155(state) => state,
        // };
        // erc1155
        //     .tokens
        //     .iter()
        //     // find all existing shuffles and nfts attached to `nft_base_id`
        //     .filter(|(id, _)| {
        //         // splitted_nft_id(id).map_or(false, |(_, _, base_id)| base_id == self.base_id)
        //         parse_token_id(id).map_or(false, |(_, base_id)| base_id == self.base_id)
        //     })
        //     // // find whether at least one of these tokens isn't an nft
        //     // .find(|(_, token)| {
        //     //     token
        //     //         .balances
        //     //         .iter()
        //     //         .map(|(_, balance)| balance.value)
        //     //         .reduce(|sum, balance| sum + balance)
        //     //         .unwrap_or(0)
        //     //         != 1
        //     // })
        //     // .map_or(Ok(()), |(id, _)| {
        //     //     Err(ContractError::TokenIsNotAnNFT(id.to_string()))
        //     // })?;

        attach_royalties_internal(&self).await?;

        Ok(HandlerResult::None(state))
    }
}
