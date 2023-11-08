use std::collections::HashMap;

use async_recursion::async_recursion;

use serde::{Deserialize, Serialize};
use warp_scarcity::{
    action::{Action, ActionResult, Configure},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable,
    contract_utils::{foreign_call::ForeignContractCaller, js_imports::SmartWeave},
    state::State,
    utils::{is_op, is_super_op},
};

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

    // if state.settings.paused
    //     && std::mem::discriminant(&action)
    //         != std::mem::discriminant(&Action::Configure(Configure::default()))
    // {
    //     return Err(ContractError::ContractIsPaused);
    // }

    if State::settings().paused().get().await
        && std::mem::discriminant(&action)
            != std::mem::discriminant(&Action::Configure(Configure::default()))
    {
        return Err(ContractError::ContractIsPaused);
    }

    // NOTE: Currently, only Pianity is allowed to transfer NFTs
    if !is_op(&direct_caller).await && !is_super_op(&direct_caller).await {
        return Err(ContractError::UnauthorizedAddress(direct_caller));
    }

    match action {
        Action::Initialize(_) => Err(ContractError::ContractAlreadyInitialized),
        Action::GetAttachedRoylaties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::AttachRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::EditAttachedRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::Transfer(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Configure(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Evolve(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Batch(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::MintNft(action) => action.action(direct_caller, state, foreign_caller).await,
    }
}
