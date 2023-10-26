use async_trait::async_trait;

use warp_scarcity::action::{ActionResult, Evolve, HandlerResult};
use warp_scarcity::error::ContractError;
use warp_scarcity::state::Parameters;

use crate::{
    actions::AsyncActionable, contract_utils::foreign_call::ForeignContractCaller,
    utils::is_super_op,
};

#[async_trait(?Send)]
impl AsyncActionable for Evolve {
    async fn action(
        self,
        caller: String,
        mut state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if !state.can_evolve {
            Err(ContractError::EvolveNotAllowed)
        } else if !is_super_op(&caller).await {
            Err(ContractError::OnlyOwnerCanEvolve)
        } else {
            state.evolve = Option::from(self.value);
            Ok(HandlerResult::Write(state))
        }
    }
}
