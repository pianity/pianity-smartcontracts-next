use async_trait::async_trait;
use warp_erc1155::action::{ActionResult, HandlerResult, Mint};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{State, Token};
use wasm_bindgen::JsValue;

use crate::contract_utils::js_imports::{Kv, Transaction};
use crate::utils::is_op;

use super::{Actionable, AsyncActionable};

fn get_token_id(prefix: Option<String>, base_id: Option<String>) -> String {
    let base_id = base_id.unwrap_or_else(Transaction::id);
    prefix.map_or(base_id.clone(), |prefix| format!("{}-{}", prefix, base_id))
}

#[async_trait(?Send)]
impl Actionable for Mint {
    fn action(self, caller: String, mut state: State) -> ActionResult {
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

        state
            .tokens
            .entry(token_id.clone())
            .or_insert(Token {
                ticker: format!("{}{}", state.default_token, state.ticker_nonce),
                tx_id: Some(Transaction::id()),
                ..Default::default()
            })
            .balances
            .entry(caller.clone())
            .or_default()
            .value += self.qty.value;

        state.ticker_nonce += 1;

        Ok(HandlerResult::Write(state))
    }
}
