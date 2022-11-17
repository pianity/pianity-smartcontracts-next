use warp_erc1155::{
    action::{ActionResult, BalanceOf, HandlerResult, ReadResponse},
    error::ContractError,
    state::State,
};

use super::Actionable;

impl Actionable for BalanceOf {
    fn action(self, _caller: String, state: State) -> ActionResult {
        let token_id = self.token_id.as_ref().unwrap_or(&state.default_token);

        let balance = {
            let token = match state.tokens.get(token_id) {
                Some(token) => token,
                None => return Err(ContractError::TokenNotFound(token_id.clone())),
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
