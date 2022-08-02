use warp_erc1155::action::{ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{Balance, State};

use crate::contract_utils::js_imports::{SmartWeave, Transaction};
use crate::utils::is_op;

use super::{approval::is_approved_for_all_impl, Actionable};

impl Actionable for Transfer {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.qty == 0 {
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

        if from != caller {
            if !is_approved_for_all_impl(&state, &from, &caller) {
                return Err(ContractError::UnauthorizedTransfer(from));
            }
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

        if from_balance.value < self.qty {
            return Err(ContractError::CallerBalanceNotEnough(from_balance.value));
        }

        // Update caller balance
        token
            .balances
            .insert(from, Balance::new(from_balance.value - self.qty));

        // Update target balance
        let target_balance = *token.balances.get(&self.to).unwrap_or(&Balance::new(0));
        token
            .balances
            .insert(self.to, Balance::new(target_balance.value + self.qty));

        Ok(HandlerResult::Write(state))
    }
}
