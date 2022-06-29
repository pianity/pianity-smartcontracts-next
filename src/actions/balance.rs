use crate::action::{ActionResult, QueryResponseMsg::Balance};
use crate::contract_utils::handler_result::HandlerResult::Read;
use crate::error::ContractError;
use crate::state::State;

pub fn balance_of(state: &State, token_id: String, target: String) -> ActionResult {
    let token = match state.tokens.get(&token_id) {
        Some(token) => token,
        None => return Err(ContractError::TokenNotFound(token_id)),
    };

    let balance = match token.balances.get(&target) {
        Some(balance) => balance,
        None => &0,
    };

    Ok(Read(Balance {
        balance: *balance,
        ticker: token.ticker.to_string(),
        target,
    }))
}
