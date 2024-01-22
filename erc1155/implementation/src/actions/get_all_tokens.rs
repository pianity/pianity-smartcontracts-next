use std::collections::HashMap;

use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, GetAllTokens, HandlerResult, ReadResponse},
    state::{Balance as StateBalance, Parameters, Token as StateToken},
};

use crate::actions::AsyncActionable;

use crate::state::State;

#[async_trait(?Send)]
impl AsyncActionable for GetAllTokens {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        let kv_tokens = State::list_tokens().await;

        let mut tokens = Vec::new();
        for (token_id, kv_token) in kv_tokens {
            let token =
                StateToken {
                    ticker: kv_token.ticker().get().await,
                    tx_id: kv_token.tx_id().get().await,
                    balances: HashMap::from_iter(kv_token.list_balances().await.iter().map(
                        |(address, balance)| (address.clone(), StateBalance::new(balance.value)),
                    )),
                };
            tokens.push((token_id, token));
        }

        Ok(HandlerResult::Read(
            state,
            ReadResponse::GetAllTokens(tokens),
        ))
    }
}
