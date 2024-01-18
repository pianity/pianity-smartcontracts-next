use async_trait::async_trait;

use warp_erc1155::action::{ActionResult, Configure, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::Parameters;

use crate::state::KvState;
use crate::{
    actions::AsyncActionable,
    utils::{is_op, is_super_op},
};

#[async_trait(?Send)]
impl AsyncActionable for Configure {
    async fn action(self, caller: String, mut state: Parameters) -> ActionResult {
        let is_super_op = is_super_op(&caller).await;
        let is_op = is_op(&caller).await;

        if !is_op
            || (self.super_operators.is_some() && !is_super_op)
            || (self.operators.is_some() && !is_super_op)
            || (self.can_evolve.is_some() && !is_super_op)
            || (self.proxies.is_some() && !is_super_op)
        {
            return Err(ContractError::UnauthorizedConfiguration);
        }

        if let Some(super_operators) = self.super_operators {
            KvState::settings()
                .super_operators()
                .set(&super_operators)
                .await;
        }

        if let Some(operators) = self.operators {
            KvState::settings().operators().set(&operators).await;
        }

        if let Some(can_evolve) = self.can_evolve {
            state.can_evolve = can_evolve;
        }

        if let Some(proxies) = self.proxies {
            KvState::settings().proxies().set(&proxies).await;
        }

        if let Some(paused) = self.paused {
            KvState::settings().paused().set(&paused).await;
        }

        if let Some(allow_free_transfer) = self.allow_free_transfer {
            KvState::settings()
                .allow_free_transfer()
                .set(&allow_free_transfer)
                .await;
        }

        if let Some(can_evolve) = self.can_evolve {
            Ok(HandlerResult::Write(state))
        } else {
            Ok(HandlerResult::None(state))
        }
    }
}
