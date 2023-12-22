use async_trait::async_trait;
use warp_lock::{
    action::{ActionResult, GetVault, HandlerResult, ReadResponse},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable, contract_utils::foreign_call::ForeignContractCaller, state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for GetVault {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let vault = State::vault(&self.owner)
            .ok_or(ContractError::OwnerHasNoVault(self.owner.clone()))
            .await?;

        let vault = vault.get().await;

        Ok(HandlerResult::Read(
            state,
            ReadResponse::GetVault((self.owner, vault)),
        ))
    }
}
