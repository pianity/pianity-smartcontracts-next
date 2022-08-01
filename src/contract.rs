use async_recursion::async_recursion;

use crate::action::{Action, ActionResult, Actionable};
use crate::actions::approval::{is_approved_for_all, set_approval_for_all};
use crate::actions::balance::balance_of;
use crate::actions::batch::batch;
use crate::actions::configure::configure;
use crate::actions::evolve::evolve;
use crate::actions::foreign_read::foreign_read;
use crate::actions::foreign_write::foreign_write;
use crate::actions::mint::mint;
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::{log, Block, Contract, SmartWeave, Transaction};
use crate::error::ContractError;
use crate::state::State;
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
            balance_of(state, original_caller, token_id, target)
        }

        Action::Configure(args) => configure(state, original_caller, args),

        Action::Evolve { value } => evolve(state, original_caller, value),

        Action::SetApprovalForAll { operator, approved } => {
            set_approval_for_all(state, original_caller, operator, approved)
        }

        Action::IsApprovedForAll { operator, owner } => {
            is_approved_for_all(state, original_caller, operator, owner)
        }

        Action::Mint(args) => mint(state, original_caller, args),

        Action::Batch(args) => batch(state, args).await,
    }
}
