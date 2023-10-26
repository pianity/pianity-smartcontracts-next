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
        mut state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if !State::all_attached_royalties(&self.base_id).exists().await {
            return Err(ContractError::TokenNotFound(self.base_id));
        }

        attach_royalties_internal(&AttachRoyalties {
            base_id: self.base_id,
            royalties: self.royalties,
            rate: self.rate,
        })
        .await?;

        Ok(HandlerResult::None(state))
    }
}
