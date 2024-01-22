use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, BalanceOf, HandlerResult, ReadResponse},
    error::ContractError,
    state::{Balance as StateBalance, Parameters},
};

use crate::actions::AsyncActionable;

use crate::state::{Balance, State};

#[async_trait(?Send)]
impl AsyncActionable for BalanceOf {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        let token_id = self
            .token_id
            .unwrap_or(State::settings().default_token().get().await);

        let balance = State::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?
            .balances(&self.target)
            .peek()
            .await
            .unwrap_or(Balance::new(0));

        Ok(HandlerResult::Read(
            state,
            ReadResponse::BalanceOf {
                balance: StateBalance::new(balance.value),
                target: self.target,
            },
        ))
    }
}
