use warp_erc1155::action::{ActionResult, Burn, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::{actions::Actionable, utils::is_op};

impl Actionable for Burn {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if !is_op(&state, &caller) {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        let owner = if let Some(owner) = self.owner {
            owner.clone()
        } else {
            caller
        };

        let token_id = self.token_id.as_ref().unwrap_or(&state.default_token);

        let balances = if let Some(token) = state.tokens.get_mut(token_id) {
            &mut token.balances
        } else {
            return Err(ContractError::TokenNotFound(token_id.clone()));
        };

        let owner_balance = if let Some(balance) = balances.get_mut(&owner) {
            balance
        } else {
            return Err(ContractError::OwnerBalanceNotEnough(owner));
        };

        if owner_balance.value < self.qty.value {
            return Err(ContractError::OwnerBalanceNotEnough(owner));
        } else if owner_balance.value - self.qty.value == 0 {
            balances.remove(&owner);

            if balances.len() == 0 {
                state.tokens.remove(token_id);
            }
        } else {
            owner_balance.value -= self.qty.value;
        }

        Ok(HandlerResult::Write(state))
    }
}
