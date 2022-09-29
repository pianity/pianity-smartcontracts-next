use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult};
use warp_erc1155::state::State;

use crate::actions::{Actionable, *};
use crate::contract_utils::js_imports::{SmartWeave, Transaction};
use crate::utils::is_op;

#[async_recursion(?Send)]
pub async fn handle(state: State, action: Action) -> ActionResult {
    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    let effective_caller = if state.settings.proxies.contains(&direct_caller) {
        original_caller
    } else {
        direct_caller
    };

    match action {
        Action::Transfer(action) => action.action(effective_caller, state),
        Action::BalanceOf(action) => action.action(effective_caller, state),
        Action::Configure(action) => action.action(effective_caller, state),
        Action::Evolve(action) => action.action(effective_caller, state),
        Action::SetApprovalForAll(action) => action.action(effective_caller, state),
        Action::IsApprovedForAll(action) => action.action(effective_caller, state),
        Action::Mint(action) => action.action(effective_caller, state),
        Action::Batch(action) => action.action(effective_caller, state).await,
    }
}
