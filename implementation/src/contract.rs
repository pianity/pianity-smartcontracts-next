use async_recursion::async_recursion;
use warp_erc1155::action::{Action, ActionResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::actions::{self, Actionable};
use crate::contract_utils::js_imports::{log, Block, Contract, SmartWeave, Transaction};
use crate::utils::is_op;

#[async_recursion]
pub async fn handle(state: State, action: Action) -> ActionResult {
    // for vrf-compatible interactions
    /*log(&("Vrf::value()".to_owned() + &Vrf::value()));
    log(&("Vrf::randomInt()".to_owned() + &Vrf::randomInt(7).to_string()));*/

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
        Action::Transfer(transfer) => transfer.action(original_caller, state),

        Action::BalanceOf { token_id, target } => {
            actions::balance_of(state, original_caller, token_id, target)
        }

        Action::Configure(args) => actions::configure(state, original_caller, args),

        Action::Evolve { value } => actions::evolve(state, original_caller, value),

        Action::SetApprovalForAll { operator, approved } => {
            actions::set_approval_for_all(state, original_caller, operator, approved)
        }

        Action::IsApprovedForAll { operator, owner } => {
            actions::is_approved_for_all(state, original_caller, operator, owner)
        }

        Action::Mint(args) => actions::mint(state, original_caller, args),

        Action::Batch(args) => actions::batch(state, args).await,
    }
}
