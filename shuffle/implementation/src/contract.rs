use std::collections::HashMap;

use async_recursion::async_recursion;

use warp_shuffle::{
    action::{Action, ActionResult},
    error::ContractError,
    state::State,
};

use crate::contract_utils::{
    foreign_call::ForeignContractCaller,
    js_imports::{log, Block, Contract, SmartWeave, Transaction},
};
use crate::{
    actions::{self, Actionable, AsyncActionable},
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
        Action::OpenShuffleBatch(action) => {
            todo!()
            // action.action(direct_caller, state, foreign_caller).await
        }
        Action::Configure(action) => action.action(direct_caller, state, foreign_caller),
        Action::Evolve(action) => action.action(direct_caller, state, foreign_caller),
        Action::Batch(action) => action.action(direct_caller, state, foreign_caller).await,
    }
}
