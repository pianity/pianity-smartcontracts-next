use std::collections::HashMap;

use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::actions::{self, Actionable, *};
use crate::contract_utils::js_imports::{log, Block, Contract, SmartWeave, Transaction};
use crate::utils::is_op;

// TODO: Remove me if this implementation idea is discarded
// #[derive(Default)]
// pub struct Executor<T: core::hash::Hash + PartialEq + Eq + Default> {
//     actions: HashMap<T, ActionHandler>,
// }
//
// impl<T: core::hash::Hash + PartialEq + Eq + Default> Executor<T> {
//     fn new() -> Self {
//         Self::default()
//     }
//
//     fn register(mut self, action: T, action_handler: ActionHandler, proxiable: bool) -> Self {
//         self.actions.insert(action, action_handler);
//
//         self
//     }
// }

#[async_recursion(?Send)]
pub async fn handle(state: State, action: Action) -> ActionResult {
    // Executor::new().register(Transfer::default(), transfer, false);

    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    match action {
        Action::Batch(_) => (),
        Action::Transfer(_) => {
            if original_caller != direct_caller
                && !state.settings.transfer_proxies.contains(&direct_caller)
            {
                return Err(ContractError::UnauthorizedAddress(direct_caller));
            }

            if !(!state.settings.transfer_proxies.is_empty()
                && (state.settings.transfer_proxies.contains(&direct_caller)
                    || is_op(&state, &direct_caller)))
            {
                return Err(ContractError::UnauthorizedAddress(original_caller));
            }
        }
        _ => {
            if original_caller != direct_caller {
                return Err(ContractError::UnauthorizedAddress(direct_caller));
            }
        }
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
