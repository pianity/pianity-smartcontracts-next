use async_recursion::async_recursion;

use warp_fee::{
    action::{Action, ActionResult},
    error::ContractError,
    state::State,
};

// use crate::actions::approval::{is_approved_for_all, set_approval_for_all};
use crate::contract_utils::js_imports::{log, Block, Contract, SmartWeave, Transaction};
use crate::{
    actions::{self, Actionable, AsyncActionable},
    utils::{is_op, is_super_op},
};

#[async_recursion(?Send)]
pub async fn handle(state: State, action: Action) -> ActionResult {
    // if !state.settings.authorized_addresses.is_empty()
    //     && !state
    //         .settings
    //         .authorized_addresses
    //         .contains(&Transaction::owner())
    // {
    //     return Err(ContractError::UnauthorizedAddress(Transaction::owner()));
    // }

    // let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    // NOTE: Currently, only Pianity is allowed to transfer NFTs
    if !is_op(&state, &direct_caller) && !is_super_op(&state, &direct_caller) {
        return Err(ContractError::UnauthorizedAddress(direct_caller));
    }

    match action {
        Action::CreateFee(action) => action.action(direct_caller, state).await,

        Action::Transfer(action) => action.action(direct_caller, state).await,

        Action::Configure(action) => action.action(direct_caller, state),

        Action::Evolve(action) => action.action(direct_caller, state),

        // Action::SetApprovalForAll { operator, approved } => {
        //     set_approval_for_all(state, operator, approved)
        // }

        // Action::IsApprovedForAll { operator, owner } => is_approved_for_all(state, operator, owner),
        Action::Batch(action) => action.action(direct_caller, state).await,
    }
}

// #[async_recursion(?Send)]
// pub async fn handle(state: State, action: Action) -> ActionResult {
//     let original_caller = Transaction::owner();
//     let direct_caller = SmartWeave::caller();
//
//     if original_caller != direct_caller
//         && !state.settings.authorized_addresses.contains(&direct_caller)
//     {
//         return Err(ContractError::UnauthorizedAddress(direct_caller));
//     }
//
//     if !state.settings.authorized_addresses.is_empty()
//         && (!state.settings.authorized_addresses.contains(&direct_caller)
//             || is_op(&state, &direct_caller))
//     {
//         return Err(ContractError::UnauthorizedAddress(original_caller));
//     }
//
//     match action {
//         Action::Transfer(action) => action.action(original_caller, state),
//         Action::BalanceOf(action) => action.action(original_caller, state),
//         Action::Configure(action) => action.action(original_caller, state),
//         Action::Evolve(action) => action.action(original_caller, state),
//         Action::SetApprovalForAll(action) => action.action(original_caller, state),
//         Action::IsApprovedForAll(action) => action.action(original_caller, state),
//         Action::Mint(action) => action.action(original_caller, state),
//         Action::Batch(action) => action.action(original_caller, state).await,
//     }
// }
