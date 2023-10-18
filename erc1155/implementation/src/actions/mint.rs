use async_trait::async_trait;
use wasm_bindgen::JsValue;

use warp_erc1155::{
    action::{ActionResult, HandlerResult, Mint},
    error::ContractError,
    state::State,
};

use crate::{
    contract_utils::js_imports::{Kv, Transaction},
    state::{KvState, Token},
    utils::is_op,
};

use super::AsyncActionable;

fn get_token_id(prefix: Option<String>, base_id: Option<String>) -> String {
    let base_id = base_id.unwrap_or_else(Transaction::id);
    prefix.map_or(base_id.clone(), |prefix| format!("{}-{}", prefix, base_id))
}

#[async_trait(?Send)]
impl AsyncActionable for Mint {
    async fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        if !(is_op(&state, &caller)) {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        let token_id = get_token_id(self.prefix, self.base_id);

        token_id.chars().all(|c| c.is_alphanumeric() || c == '-');

        // Kv::put(
        //     &format!("tokens.{}.ticker", token_id),
        //     JsValue::from_serde(&format!("{}{}", state.default_token, state.ticker_nonce)).unwrap(),
        // )
        // .await;
        //
        // Kv::put(&format!("tokens.{}.txId", token_id), Transaction::id()).await;
        //
        // Kv::put(
        //     &format!("tokens.{}.balances.{}", token_id, caller),
        //     JsValue::from(self.qty.value.to_string()),
        // )
        // .await;

        let default_token = KvState::settings().default_token().get().await;
        let ticker_nonce = KvState::settings().ticker_nonce().get().await;

        KvState::tokens(&token_id)
            .init(Token {
                ticker: format!("{}{}", default_token, ticker_nonce),
                tx_id: Some(Transaction::id()),
                ..Default::default()
            })
            .await
            .balances(&caller)
            .init_default()
            .await
            .map(|mut balances| {
                balances.value += self.qty.value;
                balances
            })
            .await;

        KvState::settings()
            .ticker_nonce()
            .map(|nonce| nonce + 1)
            .await;

        // state
        //     .tokens
        //     .entry(token_id.clone())
        //     .or_insert(Token {
        //         ticker: format!("{}{}", state.default_token, state.ticker_nonce),
        //         tx_id: Some(Transaction::id()),
        //         ..Default::default()
        //     })
        //     .balances
        //     .entry(caller.clone())
        //     .or_default()
        //     .value += self.qty.value;

        // state.ticker_nonce += 1;

        Ok(HandlerResult::Write(state))
    }
}
