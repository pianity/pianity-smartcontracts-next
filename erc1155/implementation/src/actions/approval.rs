use std::collections::HashMap;

use warp_erc1155::action::ActionResult;
use warp_erc1155::action::HandlerResult;
use warp_erc1155::action::IsApprovedForAll;
use warp_erc1155::action::ReadResponse;
use warp_erc1155::action::SetApprovalForAll;
use warp_erc1155::state::State;

use super::Actionable;

pub fn is_approved_for_all_internal(state: &State, operator: &str, owner: &str) -> bool {
    match state.approvals.get(owner) {
        Some(approved_ops) => approved_ops.get(operator).unwrap_or(&false).to_owned(),
        None => false,
    }
}

impl Actionable for IsApprovedForAll {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        let approved = is_approved_for_all_internal(&state, &self.operator, &self.owner);

        Ok(HandlerResult::Read(
            state,
            ReadResponse::ApprovedForAll {
                approved,
                owner: self.owner,
                operator: self.operator,
            },
        ))
    }
}

impl Actionable for SetApprovalForAll {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if let Some(approved_ops) = state.approvals.get_mut(&caller) {
            approved_ops.insert(self.operator, self.approved);
        } else {
            let mut approved_ops = HashMap::new();
            approved_ops.insert(self.operator, self.approved);

            state.approvals.insert(caller, approved_ops);
        };

        Ok(HandlerResult::Write(state))
    }
}
