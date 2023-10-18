use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, BalanceOf, HandlerResult, ReadResponse},
    error::ContractError,
    state::State,
};

use crate::actions::{Actionable, AsyncActionable};

use crate::state::{Balance, KvState};

#[async_trait(?Send)]
impl AsyncActionable for BalanceOf {
    async fn action(self, _caller: String, state: State) -> ActionResult {
        let token_id = self
            .token_id
            .unwrap_or(KvState::settings().default_token().get().await);

        let balance = KvState::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?
            .balances(&self.target)
            .peek()
            .await
            .unwrap_or(Balance::new(0))
            .value;

        // let token_id = self.token_id.as_ref().unwrap_or(&state.default_token);
        //
        // let balance = {
        //     let token = match state.tokens.get(token_id) {
        //         Some(token) => token,
        //         None => return Err(ContractError::TokenNotFound(token_id.clone())),
        //     };
        //
        //     match token.balances.get(&self.target) {
        //         Some(balance) => balance.clone().value,
        //         None => 0,
        //     }
        // };

        Ok(HandlerResult::Read(
            state,
            ReadResponse::Balance {
                balance,
                target: self.target,
            },
        ))
    }
}
