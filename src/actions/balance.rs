use crate::action::{ActionResult, ReadResponse::Balance};
use crate::contract_utils::handler_result::HandlerResult::Read;
use crate::error::ContractError;
use crate::state::State;

pub fn balance_of(state: State, token_id: String, target: String) -> ActionResult {
    let balance = {
        let token = match state.tokens.get(&token_id) {
            Some(token) => token,
            None => return Err(ContractError::TokenNotFound(token_id)),
        };

        match token.balances.get(&target) {
            Some(balance) => balance.clone(),
            None => 0,
        }
    };

    Ok(Read(state, Balance { balance, target }))
}
