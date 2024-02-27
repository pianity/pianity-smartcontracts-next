use async_trait::async_trait;

use warp_erc1155::action::ActionResult;
use warp_erc1155::action::HandlerResult;
use warp_erc1155::action::IsApprovedForAll;
use warp_erc1155::action::ReadResponse;
use warp_erc1155::action::SetApprovalForAll;
use warp_erc1155::state::Parameters;

use crate::{actions::AsyncActionable, state::State};

pub async fn is_approved_for_all_internal(operator: &str, owner: &str) -> bool {
    if operator == owner {
        true
    } else {
        State::approvals(owner)
            .peek()
            .approves(operator)
            .await
            .unwrap_or(false)
    }
}

#[async_trait(?Send)]
impl AsyncActionable for IsApprovedForAll {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        let approved = is_approved_for_all_internal(&self.operator, &self.owner).await;

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
    async fn action(self, caller: String, state: Parameters) -> ActionResult {
        State::approvals(&caller)
            .init_default()
            .await
            .approves(&self.operator)
            .set(&self.approved)
            .await;

        Ok(HandlerResult::None(state))
    }
}
