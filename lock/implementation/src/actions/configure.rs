use async_trait::async_trait;
use warp_lock::{
    action::{ActionResult, Configure, HandlerResult},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::Actionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::State,
    utils::{is_op, is_super_op},
};

use super::AsyncActionable;

#[async_trait(?Send)]
impl AsyncActionable for Configure {
    async fn action(
        self,
        caller: String,
        mut state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let is_super_op = is_super_op(&caller).await;
        let is_op = is_op(&caller).await;

        if !is_op
            || (self.super_operators.is_some() && !is_super_op)
            || (self.operators.is_some() && !is_super_op)
            || (self.can_evolve.is_some() && !is_super_op)
        {
            return Err(ContractError::UnauthorizedConfiguration);
        }

        if let Some(super_operators) = self.super_operators {
            State::settings()
                .super_operators()
                .set(&super_operators)
                .await;
        }

        if let Some(operators) = self.operators {
            State::settings().operators().set(&operators).await;
        }

        if let Some(paused) = self.paused {
            State::settings().paused().set(&paused).await;
        }

        if let Some(erc1155) = self.erc1155 {
            State::settings().erc1155().set(&erc1155).await;
        }

        return Ok(HandlerResult::None(state));
    }
}
