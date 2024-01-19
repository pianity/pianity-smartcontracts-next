use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, GetRoyalties, HandlerResult, ReadResponse},
    error::ContractError,
    state::{AttachedRoyalties as AttachedRoyaltiesState, Parameters},
};

use crate::{
    actions::AsyncActionable, contract_utils::foreign_call::ForeignContractCaller, state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for GetRoyalties {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let attached_royalties = State::all_attached_royalties(&self.base_id)
            .ok_or(ContractError::TokenNotFound(self.base_id.clone()))
            .await?
            .get()
            .await;

        Ok(HandlerResult::Read(
            state,
            ReadResponse::GetRoyalties((
                self.base_id,
                AttachedRoyaltiesState {
                    base_id: attached_royalties.base_id,
                    royalties: attached_royalties.royalties,
                    rate: attached_royalties.rate,
                },
            )),
        ))
    }
}
