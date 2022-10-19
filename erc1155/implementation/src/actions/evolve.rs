use warp_erc1155::action::{ActionResult, Evolve, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::utils::is_super_op;

use super::Actionable;

impl Actionable for Evolve {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        match state.can_evolve {
            Some(can_evolve) => {
                if can_evolve && is_super_op(&state, &caller) {
                    state.evolve = Option::from(self.value);
                    Ok(HandlerResult::Write(state))
                } else {
                    Err(ContractError::OnlyOwnerCanEvolve)
                }
            }
            None => Err(ContractError::EvolveNotAllowed),
        }
    }
}
