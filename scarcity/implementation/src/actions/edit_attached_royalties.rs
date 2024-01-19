use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, AttachRoyalties, EditAttachedRoyalties, HandlerResult},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::{attach_royalties_internal, AsyncActionable},
    contract_utils::foreign_call::ForeignContractCaller,
    state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for EditAttachedRoyalties {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let old_royalties = State::all_attached_royalties(&self.base_id)
            .peek()
            .await
            .ok_or_else(|| ContractError::RoyaltiesNotFound(self.base_id.clone()))?;

        let new_royalties = {
            let new_royalties = self
                .royalties
                .unwrap_or_else(|| old_royalties.royalties.clone());
            let new_rate = self.rate.unwrap_or(old_royalties.rate);

            if new_royalties != old_royalties.royalties || new_rate != old_royalties.rate {
                Some(AttachRoyalties {
                    base_id: self.base_id,
                    royalties: new_royalties,
                    rate: new_rate,
                })
            } else {
                None
            }
        };

        if let Some(new_royalties) = new_royalties {
            attach_royalties_internal(&new_royalties).await?;
        } else {
            return Err(ContractError::RoyaltiesUnchanged);
        }

        Ok(HandlerResult::None(state))
    }
}
