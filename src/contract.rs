use crate::action::{Action, ActionResult, QueryResponseMsg};
use crate::actions::approval::{is_approved_for_all, set_approval_for_all};
use crate::actions::balance::balance_of;
use crate::actions::batch::batch;
use crate::actions::configure::configure;
use crate::actions::evolve::evolve;
use crate::actions::foreign_read::foreign_read;
use crate::actions::foreign_write::foreign_write;
use crate::actions::transfer::transfer;
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::{log, Block, Contract, SmartWeave, Transaction};
use crate::error::ContractError;
use crate::state::State;

pub async fn handle(state: State, action: Action) -> ActionResult {
    // for vrf-compatible interactions
    /*log(&("Vrf::value()".to_owned() + &Vrf::value()));
    log(&("Vrf::randomInt()".to_owned() + &Vrf::randomInt(7).to_string()));*/

    if !state.settings.authorized_addresses.is_empty()
        && !state
            .settings
            .authorized_addresses
            .contains(&Transaction::owner())
    {
        return Err(ContractError::UnauthorizedAddress(Transaction::owner()));
    }

    match action {
        Action::Transfer {
            from,
            to,
            token_id,
            qty,
        } => transfer(state, from, to, token_id, qty),

        Action::BalanceOf { token_id, target } => balance_of(state, token_id, target),
        // Action::Configure(args) => configure(state, args),
        //
        //
        // Action::Evolve { value } => evolve(state, value),
        //
        // Action::SetApprovalForAll { operator, approved } => {
        //     set_approval_for_all(state, operator, approved)
        // }
        //
        // Action::IsApprovedForAll { operator, owner } => is_approved_for_all(state, operator, owner),
        //
        // Action::Batch(args) => batch(state, args),
        //
        // Action::ForeignRead { contract_tx_id } => foreign_read(state, contract_tx_id).await,
        //
        // Action::ForeignWrite {
        //     contract_tx_id,
        //     qty,
        //     target,
        // } => foreign_write(state, contract_tx_id, qty, target).await,
    }
}
