use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, HandlerResult, RemoveAttachedRoyalties},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::{AsyncActionable},
    contract_utils::foreign_call::ForeignContractCaller,
    state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for RemoveAttachedRoyalties {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if !State::all_attached_royalties(&self.base_id).exists().await {
            return Err(ContractError::RoyaltiesNotFound(self.base_id));
        }

        State::delete_all_attached_royalties(&self.base_id).await;

        Ok(HandlerResult::None(state))
    }
}
