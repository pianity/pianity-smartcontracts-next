use async_recursion::async_recursion;

use warp_lock::{
    action::{Action, ActionResult},
    error::ContractError,
    state::State,
};

use crate::contract_utils::js_imports::{log, SmartWeave};
use crate::{
    actions::{self, Actionable, AsyncActionable},
    utils::{is_op, is_super_op},
};

#[async_recursion(?Send)]
pub async fn handle(state: State, action: Action) -> ActionResult {
    // let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    match action {
        Action::TransferLocked(action) => action.action(direct_caller, state).await,
        Action::Unlock(action) => action.action(direct_caller, state).await,
        Action::Configure(action) => action.action(direct_caller, state),
        Action::Evolve(action) => action.action(direct_caller, state),
        Action::Batch(action) => action.action(direct_caller, state).await,
    }
}
