use crate::action::{Action, ActionResult, BatchArgs};
use crate::contract::handle;
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::State;

use super::approval::is_approved_for_all_impl;

pub async fn batch(mut state: &mut State, args: BatchArgs) -> ActionResult {
    for action in &args.actions {
        if let Action::Batch(_) = action {
            return Err(ContractError::ForbiddenNestedBatch);
        }
    }

    let mut results = Vec::new();

    for action in args.actions {
        let state = if let HandlerResult::Read(result) = handle(state, action).await? {
            results.push(result);
        };
    }

    Ok(HandlerResult::Write)
}
