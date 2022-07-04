use crate::action::ActionResult;
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::State;

use super::approval::is_approved_for_all_impl;

pub fn transfer(
    mut state: State,
    from: Option<String>,
    to: String,
    token_id: String,
    qty: u64,
) -> ActionResult {
    let caller = Transaction::owner();

    if qty == 0 {
        return Err(ContractError::TransferAmountMustBeHigherThanZero);
    }

    let from = if let Some(from) = from {
        from
    } else {
        Transaction::owner()
    };

    if from != caller {
        if !is_approved_for_all_impl(&state, &from, &caller) {
            return Err(ContractError::UnauthorizedTransfer(from));
        }
    }

    if from == to {
        return Err(ContractError::TransferFromAndToCannotBeEqual);
    }

    let token = if let Some(token) = state.tokens.get_mut(&token_id) {
        token
    } else {
        return Err(ContractError::TokenNotFound(token_id));
    };

    // Checking if caller has enough funds
    let from_balance = *token.balances.get(&from).unwrap_or(&0);
    if from_balance < qty {
        return Err(ContractError::CallerBalanceNotEnough(from_balance));
    }

    // Update caller balance
    token.balances.insert(from, from_balance - qty);

    // Update target balance
    let target_balance = *token.balances.get(&to).unwrap_or(&0);
    token.balances.insert(to, target_balance + qty);

    Ok(HandlerResult::Write(state))
}
