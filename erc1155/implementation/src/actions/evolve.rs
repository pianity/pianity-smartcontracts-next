use async_trait::async_trait;

use warp_erc1155::action::{ActionResult, Evolve, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::Parameters;

use crate::{actions::AsyncActionable, utils::is_super_op};

#[async_trait(?Send)]
impl AsyncActionable for Evolve {
    async fn action(self, caller: String, mut state: Parameters) -> ActionResult {
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
