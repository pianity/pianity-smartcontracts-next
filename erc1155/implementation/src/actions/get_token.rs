use std::collections::HashMap;

use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, GetToken, HandlerResult, ReadResponse},
    error::ContractError,
    state::{Balance as StateBalance, Parameters, Token as StateToken},
};

use crate::actions::AsyncActionable;

use crate::state::KvState;

#[async_trait(?Send)]
impl AsyncActionable for GetToken {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        let token_id = self
            .token_id
            .unwrap_or(KvState::settings().default_token().get().await);

        let kv_token = KvState::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?;

        let token = StateToken {
            ticker: kv_token.ticker().get().await,
            tx_id: kv_token.tx_id().get().await,
            balances: HashMap::from_iter(
                kv_token
                    .list_balances()
                    .await
                    .iter()
                    .map(|(address, balance)| (address.clone(), StateBalance::new(balance.value))),
            ),
        };

        Ok(HandlerResult::Read(state, ReadResponse::GetToken(token)))
    }
}
