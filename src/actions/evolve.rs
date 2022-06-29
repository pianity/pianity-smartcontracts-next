use crate::action::ActionResult;
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError::{EvolveNotAllowed, OnlyOwnerCanEvolve};
use crate::state::State;

pub fn evolve(state: &mut State, value: String) -> ActionResult {
    match state.can_evolve {
        Some(can_evolve) => {
            if can_evolve && state.settings.super_owner == Transaction::owner() {
                state.evolve = Option::from(value);
                Ok(HandlerResult::Write)
            } else {
                Err(OnlyOwnerCanEvolve)
            }
        }
        None => Err(EvolveNotAllowed),
    }
}
