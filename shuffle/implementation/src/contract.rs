use async_recursion::async_recursion;

use warp_shuffle::{
    action::{Action, ActionResult},
    error::ContractError,
    state::State,
};

use crate::contract_utils::{foreign_call::ForeignContractCaller, js_imports::SmartWeave};
use crate::{
    actions::{Actionable, AsyncActionable},
    utils::{is_op, is_super_op},
};

#[async_recursion(?Send)]
pub async fn handle(
    state: State,
    action: Action,
    foreign_caller: &mut ForeignContractCaller,
) -> ActionResult {
    // let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    // NOTE: Currently, only Pianity is allowed to transfer NFTs
    if !is_op(&state, &direct_caller) && !is_super_op(&state, &direct_caller) {
        return Err(ContractError::UnauthorizedAddress(direct_caller));
    }

    match action {
        Action::MintShuffle(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::OpenShuffle(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Configure(action) => action.action(direct_caller, state),
        Action::Evolve(action) => action.action(direct_caller, state),
        Action::Batch(action) => action.action(direct_caller, state, foreign_caller).await,
    }
}
