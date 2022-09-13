use warp_fee::action::{ActionResult, Evolve, HandlerResult};
use warp_fee::error::ContractError;
use warp_fee::state::State;

use crate::contract_utils::js_imports::Transaction;

use super::Actionable;

impl Actionable for Evolve {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        match state.can_evolve {
            Some(can_evolve) => {
                if can_evolve && state.settings.super_operator == Transaction::owner() {
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