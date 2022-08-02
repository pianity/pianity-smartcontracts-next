use warp_erc1155::{
    action::{ActionResult, BalanceOf, HandlerResult, ReadResponse},
    error::ContractError,
    state::State,
};

use super::Actionable;

impl Actionable for BalanceOf {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        let balance = {
            let token = match state.tokens.get(&self.token_id) {
                Some(token) => token,
                None => return Err(ContractError::TokenNotFound(self.token_id)),
            };

            match token.balances.get(&self.target) {
                Some(balance) => balance.clone().value,
                None => 0,
            }
        };

        Ok(HandlerResult::Read(
            state,
            ReadResponse::Balance {
                balance,
                target: self.target,
            },
        ))
    }
}
