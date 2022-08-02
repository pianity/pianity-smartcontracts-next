use std::collections::HashMap;

use warp_erc1155::action::ActionResult;
use warp_erc1155::action::HandlerResult;
use warp_erc1155::action::ReadResponse;
use warp_erc1155::state::State;

use crate::contract_utils::js_imports::Transaction;

pub fn is_approved_for_all_impl(state: &State, operator: &str, owner: &str) -> bool {
    match state.approvals.get(owner) {
        Some(approved_ops) => approved_ops.get(operator).unwrap_or(&false).to_owned(),
        None => false,
    }
}

pub fn is_approved_for_all(
    state: State,
    caller: String,
    operator: String,
    owner: String,
) -> ActionResult {
    let approved = is_approved_for_all_impl(&state, &operator, &owner);

    Ok(HandlerResult::Read(
        state,
        ReadResponse::ApprovedForAll {
            approved,
            owner,
            operator,
        },
    ))
}

pub fn set_approval_for_all(
    mut state: State,
    caller: String,
    operator: String,
    approved: bool,
) -> ActionResult {
    if let Some(approved_ops) = state.approvals.get_mut(&caller) {
        approved_ops.insert(operator, approved);
    } else {
        let mut approved_ops = HashMap::new();
        approved_ops.insert(operator, approved);

        state.approvals.insert(caller, approved_ops);
    };

    Ok(HandlerResult::Write(state))
}
