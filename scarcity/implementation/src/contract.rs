use async_recursion::async_recursion;

use warp_scarcity::{
    action::{Action, ActionResult},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable,
    contract_utils::{foreign_call::ForeignContractCaller, js_imports::SmartWeave},
    state::State,
    utils::{is_op, is_super_op},
};

pub fn is_action_read(action: &Action) -> bool {
    match action {
        Action::GetRoyalties(_) => true,
        Action::GetAllRoyalties(_) => true,
        _ => false,
    }
}

pub fn allowed_in_pause(action: &Action) -> bool {
    match action {
        Action::Configure(_) => true,
        _ => is_action_read(action),
    }
}

#[async_recursion(?Send)]
pub async fn handle(
    state: Parameters,
    action: Action,
    foreign_caller: &mut ForeignContractCaller,
) -> ActionResult {
    let direct_caller = SmartWeave::caller();

    if let Action::Initialize(initialize) = action {
        return initialize
            .action(direct_caller, state, foreign_caller)
            .await;
    } else if state.initial_state.is_some() {
        return Err(ContractError::ContractUninitialized);
    }

    if !allowed_in_pause(&action) && State::settings().paused().get().await {
        return Err(ContractError::ContractIsPaused);
    }

    // NOTE: Currently, only Pianity is allowed to transfer NFTs
    if !is_action_read(&action)
        && !is_op(&direct_caller).await
        && !is_super_op(&direct_caller).await
    {
        return Err(ContractError::UnauthorizedAddress(direct_caller));
    }

    match action {
        Action::Initialize(_) => Err(ContractError::ContractAlreadyInitialized),
        Action::GetRoyalties(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::GetAllRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::AttachRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::EditAttachedRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::RemoveAttachedRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::Transfer(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Configure(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Evolve(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Batch(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::MintNft(action) => action.action(direct_caller, state, foreign_caller).await,
    }
}
