use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::Parameters;

use crate::{
    actions::AsyncActionable,
    contract_utils::js_imports::{SmartWeave, Transaction},
    state::KvState,
};

pub fn allowed_in_pause(action: &Action) -> bool {
    matches!(
        action,
        Action::Configure(_)
            | Action::GetToken(_)
            | Action::GetAllTokens(_)
            | Action::BalanceOf(_)
            | Action::ReadSettings(_)
    )
}

pub async fn execute_action(
    action: Box<Action>,
    effective_caller: String,
    state: Parameters,
) -> ActionResult {
    match *action {
        Action::Initialize(_) => Err(ContractError::ContractAlreadyInitialized),
        Action::AsDirectCaller(_) => unreachable!("AsDirectCaller wasn't properly unwrapped"),
        Action::GetToken(action) => action.action(effective_caller, state).await,
        Action::GetAllTokens(action) => action.action(effective_caller, state).await,
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

#[async_recursion(?Send)]
pub async fn handle(state: Parameters, action: Action) -> ActionResult {
    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    if let Action::Initialize(initialize) = action {
        return initialize.action(direct_caller, state).await;
    } else if state.initial_state.is_some() {
        return Err(ContractError::ContractUninitialized);
    }

    if !allowed_in_pause(&action) && KvState::settings().paused().get().await {
        return Err(ContractError::ContractIsPaused);
    }

    let (effective_caller, action) = if let Action::AsDirectCaller(action) = action {
        (direct_caller, action.action)
    } else {
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

        (effective_caller, Box::new(action))
    };

    execute_action(action, effective_caller, state).await
}
