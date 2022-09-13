use warp_erc1155::action::{ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{Balance, State};

use crate::utils::is_op;

use super::{approval::is_approved_for_all_impl, Actionable};

impl Actionable for Transfer {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        let from = if let Some(from) = self.from {
            if from.len() == 0 && !is_op(&state, &caller) {
                return Err(ContractError::UnauthorizedAddress(caller));
            }

            from
        } else {
            caller.clone()
        };

        if from != caller && !is_approved_for_all_impl(&state, &caller, &from) {
            return Err(ContractError::UnauthorizedTransfer(from));
        }

        if from == self.to {
            return Err(ContractError::TransferFromAndToCannotBeEqual);
        }

        let token = if let Some(token) = state.tokens.get_mut(&self.token_id) {
            token
        } else {
            return Err(ContractError::TokenNotFound(self.token_id));
        };

        // Checking if caller has enough funds
        let from_balance = *token.balances.get(&from).unwrap_or(&Balance::new(0));

        if from_balance.value < self.qty.value {
            return Err(ContractError::CallerBalanceNotEnough(from_balance.value));
        }

        let from_new_balance = Balance::new(from_balance.value - self.qty.value);

        if from_new_balance.value == 0 {
            token.balances.remove(&from);
        } else {
            // Update caller balance
            token.balances.insert(from, from_new_balance);
        }

        // Update target balance
        let target_balance = *token.balances.get(&self.to).unwrap_or(&Balance::new(0));
        token
            .balances
            .insert(self.to, Balance::new(target_balance.value + self.qty.value));

        Ok(HandlerResult::Write(state))
    }
}
