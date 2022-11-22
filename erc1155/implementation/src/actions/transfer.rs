use warp_erc1155::action::{ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{Balance, State};

use crate::contract_utils::js_imports::log;
use crate::utils::is_op;

use super::{approval::is_approved_for_all_internal, Actionable};

impl Actionable for Transfer {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        let from = if let Some(from) = self.from {
            from
        } else {
            caller.clone()
        };

        if !state.settings.allow_free_transfer && !is_op(&state, &caller) {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        if from != caller && !is_approved_for_all_internal(&state, &caller, &from) {
            return Err(ContractError::UnauthorizedTransfer(from));
        }

        if from == self.to {
            return Err(ContractError::TransferFromAndToCannotBeEqual);
        }

        let token_id = self.token_id.as_ref().unwrap_or(&state.default_token);
        let token = state
            .tokens
            .get_mut(token_id)
            .ok_or_else(|| ContractError::TokenNotFound(token_id.clone()))?;

        // Checking if caller has enough funds
        let from_balance = *token.balances.get(&from).unwrap_or(&Balance::new(0));

        if from_balance.value < self.qty.value {
            return Err(ContractError::OwnerBalanceNotEnough(from));
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
