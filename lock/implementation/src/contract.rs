use async_recursion::async_recursion;

use warp_lock::{
    action::{Action, ActionResult, Configure},
    error::ContractError,
    state::State,
};

use crate::{
    actions::{self, Actionable, AsyncActionable},
    contract_utils::{
        foreign_call::ForeignContractCaller,
        js_imports::{log, SmartWeave},
    },
    utils::{is_op, is_super_op},
};

#[async_recursion(?Send)]
pub async fn handle(
    state: State,
    action: Action,
    foreign_caller: &mut ForeignContractCaller,
) -> ActionResult {
    let direct_caller = SmartWeave::caller();

    if state.settings.paused
        && std::mem::discriminant(&action)
            != std::mem::discriminant(&Action::Configure(Configure::default()))
    {
        return Err(ContractError::ContractIsPaused);
    }

    match action {
        Action::TransferLocked(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Unlock(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Configure(action) => action.action(direct_caller, state),
        Action::Evolve(action) => action.action(direct_caller, state),
        Action::Batch(action) => action.action(direct_caller, state, foreign_caller).await,
    }
}
