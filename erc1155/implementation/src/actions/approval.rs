use async_trait::async_trait;

use warp_erc1155::action::ActionResult;
use warp_erc1155::action::HandlerResult;
use warp_erc1155::action::IsApprovedForAll;
use warp_erc1155::action::ReadResponse;
use warp_erc1155::action::SetApprovalForAll;
use warp_erc1155::state::Parameters;

use crate::{actions::AsyncActionable, state::KvState};

// pub async fn is_approved_for_all_internal(operator: &str, owner: &str) -> bool {
//     return KvState::approvals(owner).approvals(operator).value().await;
//     // match state.approvals.get(owner) {
//     //     Some(approved_ops) => approved_ops.get(operator).unwrap_or(&false).to_owned(),
//     //     None => false,
//     // }
// }

#[async_trait(?Send)]
impl AsyncActionable for IsApprovedForAll {
    async fn action(self, caller: String, mut state: Parameters) -> ActionResult {
        // let approved = is_approved_for_all_internal(&state, &self.operator, &self.owner);
        let approved = !KvState::approvals(&self.owner)
            .peek()
            .approves(&self.operator)
            .await
            .unwrap_or(false);

        Ok(HandlerResult::Read(
            state,
            ReadResponse::IsApprovedForAll {
                approved,
                owner: self.owner,
                operator: self.operator,
            },
        ))
    }
}

#[async_trait(?Send)]
impl AsyncActionable for SetApprovalForAll {
    async fn action(self, caller: String, mut state: Parameters) -> ActionResult {
        KvState::approvals(&caller)
            .init_default()
            .await
            .approves(&self.operator)
            .set(&self.approved)
            .await;

        // if let Some(approved_ops) = state.approvals.get_mut(&caller) {
        //     approved_ops.insert(self.operator, self.approved);
        // } else {
        //     let mut approved_ops = HashMap::new();
        //     approved_ops.insert(self.operator, self.approved);
        //
        //     state.approvals.insert(caller, approved_ops);
        // };

        Ok(HandlerResult::None(state))
    }
}
