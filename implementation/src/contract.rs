use async_recursion::async_recursion;
use warp_erc1155::action::{Action, ActionResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::actions::{self, Actionable, *};
use crate::contract_utils::js_imports::{log, Block, Contract, SmartWeave, Transaction};
use crate::utils::is_op;

#[async_recursion(?Send)]
pub async fn handle(state: State, action: Action) -> ActionResult {
    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    if original_caller != direct_caller
        && !state.settings.authorized_addresses.contains(&direct_caller)
    {
        return Err(ContractError::UnauthorizedAddress(direct_caller));
    }

    if !state.settings.authorized_addresses.is_empty()
        && (!state.settings.authorized_addresses.contains(&direct_caller)
            || is_op(&state, &direct_caller))
    {
        return Err(ContractError::UnauthorizedAddress(original_caller));
    }

    match action {
        Action::Transfer(action) => action.action(original_caller, state),
        Action::BalanceOf(action) => action.action(original_caller, state),
        Action::Configure(action) => action.action(original_caller, state),
        Action::Evolve(action) => action.action(original_caller, state),
        Action::SetApprovalForAll(action) => action.action(original_caller, state),
        Action::IsApprovedForAll(action) => action.action(original_caller, state),
        Action::Mint(action) => action.action(original_caller, state),
        Action::Batch(action) => action.action(original_caller, state).await,
    }
}
