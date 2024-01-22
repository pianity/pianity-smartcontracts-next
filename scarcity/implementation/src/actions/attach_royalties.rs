use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, AttachRoyalties, HandlerResult},
    error::ContractError,
    state::{Parameters, UNIT},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::{AttachedRoyalties, State},
};

pub async fn attach_royalties_internal(
    attach_royalties: &AttachRoyalties,
) -> Result<(), ContractError> {
    if attach_royalties.rate > UNIT {
        return Err(ContractError::InvalidRate);
    }

    // Check that the sum of all royalties is equal to UNIT
    let royalties_sum = attach_royalties
        .royalties
        .values()
        .copied()
        .reduce(|sum, royalty| sum + royalty)
        .unwrap_or(0);

    if royalties_sum != UNIT {
        return Err(ContractError::InvalidRoyalties);
    }

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

        attach_royalties_internal(&self).await?;

        Ok(HandlerResult::None(state))
    }
}
