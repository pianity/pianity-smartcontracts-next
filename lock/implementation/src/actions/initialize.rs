use async_trait::async_trait;

use warp_lock::{
    action::{ActionResult, HandlerResult, Initialize},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::{Settings, State},
};

#[async_trait(?Send)]
impl AsyncActionable for Initialize {
    async fn action(
        self,
        _caller: String,
        mut parameters: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if let Some(init_state) = parameters.initial_state {
            let state = State {
                settings: Settings {
                    paused: init_state.settings.paused,
                    super_operators: init_state.settings.super_operators.clone(),
                    operators: init_state.settings.operators.clone(),
                    erc1155: init_state.settings.erc1155.clone(),
                },
                vault: init_state.vault.clone(),
            };

            State::init(&state).await;

            parameters.initial_state = None;

            Ok(HandlerResult::Write(parameters))
        } else {
            Err(ContractError::ContractAlreadyInitialized)
        }
    }
}
