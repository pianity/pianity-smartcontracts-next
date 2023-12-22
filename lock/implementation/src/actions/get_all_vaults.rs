use async_trait::async_trait;
use warp_lock::{
    action::{ActionResult, GetAllVaults, HandlerResult, ReadResponse},
    state::Parameters,
};

use crate::{
    actions::AsyncActionable, contract_utils::foreign_call::ForeignContractCaller, state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for GetAllVaults {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let vault = State::list_vault().await;

        Ok(HandlerResult::Read(
            state,
            ReadResponse::GetAllVaults(vault),
        ))
    }
}
