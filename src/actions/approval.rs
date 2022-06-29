use std::collections::HashMap;

use crate::action::ActionResult;
use crate::action::QueryResponseMsg::ApprovedForAll;
use crate::contract_utils::handler_result::HandlerResult::{Read, Write};
use crate::contract_utils::js_imports::Transaction;
use crate::state::State;

pub fn is_approved_for_all_impl(state: &State, operator: &str, owner: &str) -> bool {
    match state.approvals.get(owner) {
        Some(approved_ops) => approved_ops.get(operator).unwrap_or(&false).to_owned(),
        None => false,
    }
}

pub fn is_approved_for_all(state: &State, operator: String, owner: String) -> ActionResult {
    let approved = is_approved_for_all_impl(state, &operator, &owner);

    Ok(Read(ApprovedForAll {
        approved,
        owner,
        operator,
    }))
}

pub fn set_approval_for_all(state: &mut State, operator: String, approved: bool) -> ActionResult {
    if let Some(approved_ops) = state.approvals.get_mut(&Transaction::owner()) {
        approved_ops.insert(operator, approved);
    } else {
        let mut approved_ops = HashMap::new();
        approved_ops.insert(operator, approved);

        state.approvals.insert(Transaction::owner(), approved_ops);
    };

    Ok(Write)
}
