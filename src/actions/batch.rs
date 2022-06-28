use crate::action::{Action, ActionResult, BatchArgs};
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::State;

use super::approval::is_approved_for_all_impl;

pub fn batch(mut state: State, args: BatchArgs) -> ActionResult {
    for action in args.actions {
        if let Action::Batch(_) = action {
            return Err(ContractError::ForbiddenNestedBatch);
        }
    }

    for action in args.actions {
        return;
    }

    Ok(HandlerResult::NewState(state))
}
