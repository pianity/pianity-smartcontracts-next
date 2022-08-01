use crate::action::ActionResult;
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError::{EvolveNotAllowed, OnlyOwnerCanEvolve};
use crate::state::State;

pub fn evolve(mut state: State, caller: String, value: String) -> ActionResult {
    match state.can_evolve {
        Some(can_evolve) => {
            if can_evolve && state.settings.super_operator == Transaction::owner() {
                state.evolve = Option::from(value);
                Ok(HandlerResult::Write(state))
            } else {
                Err(OnlyOwnerCanEvolve)
            }
        }
        None => Err(EvolveNotAllowed),
    }
}
