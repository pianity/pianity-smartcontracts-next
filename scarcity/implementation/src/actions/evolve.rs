use warp_scarcity::action::{ActionResult, Evolve, HandlerResult};
use warp_scarcity::error::ContractError;
use warp_scarcity::state::State;

use crate::utils::is_super_op;

use super::Actionable;

impl Actionable for Evolve {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if !state.settings.can_evolve {
            Err(ContractError::EvolveNotAllowed)
        } else if !is_super_op(&state, &caller) {
            Err(ContractError::OnlyOwnerCanEvolve)
        } else {
            state.evolve = Option::from(self.value);
            Ok(HandlerResult::Write(state))
        }
    }
}
