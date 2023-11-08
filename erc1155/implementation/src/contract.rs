use std::collections::HashMap;

use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult, Configure, HandlerResult, Initialize};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{InitialState, Parameters};

use crate::contract_utils::js_imports::{log, KvJs};
use crate::state::{Approvals, Balance, Settings, Token};
use crate::{
    actions::{Actionable, AsyncActionable, *},
    contract_utils::js_imports::{SmartWeave, Transaction},
    state::KvState,
};

#[async_recursion(?Send)]
pub async fn handle(state: Parameters, action: Action) -> ActionResult {
    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    if let Action::Initialize(initialize) = action {
        return initialize.action(direct_caller, state).await;
    } else if state.initial_state.is_some() {
        return Err(ContractError::ContractUninitialized);
    }

    // if state.settings.paused
    //     && std::mem::discriminant(&action)
    //         != std::mem::discriminant(&Action::Configure(Configure::default()))
    // {
    //     return Err(ContractError::ContractIsPaused);
    // }

    let effective_caller = if KvState::settings()
        .proxies()
        .get()
        .await
        .contains(&direct_caller)
    {
        original_caller
    } else {
        direct_caller
    };

    match action {
        Action::Initialize(_) => Err(ContractError::ContractAlreadyInitialized),
        Action::GetToken(action) => action.action(effective_caller, state).await,
        Action::BalanceOf(action) => action.action(effective_caller, state).await,
        Action::ReadSettings(action) => action.action(effective_caller, state).await,
        Action::Transfer(action) => action.action(effective_caller, state).await,
        Action::Configure(action) => action.action(effective_caller, state).await,
        Action::Evolve(action) => action.action(effective_caller, state).await,
        Action::SetApprovalForAll(action) => action.action(effective_caller, state).await,
        Action::IsApprovedForAll(action) => action.action(effective_caller, state).await,
        Action::Mint(action) => action.action(effective_caller, state).await,
        Action::Burn(action) => action.action(effective_caller, state).await,
        Action::Batch(action) => action.action(effective_caller, state).await,
    }
}
