use warp_erc1155::action::{ActionResult, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::contract_utils::js_imports::Transaction;

pub fn evolve(mut state: State, caller: String, value: String) -> ActionResult {
    match state.can_evolve {
        Some(can_evolve) => {
            if can_evolve && state.settings.super_operator == Transaction::owner() {
                state.evolve = Option::from(value);
                Ok(HandlerResult::Write(state))
            } else {
                Err(ContractError::OnlyOwnerCanEvolve)
            }
        }
        None => Err(ContractError::EvolveNotAllowed),
    }
}
