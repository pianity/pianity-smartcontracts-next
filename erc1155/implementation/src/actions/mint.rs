use async_trait::async_trait;

use warp_erc1155::{
    action::{ActionResult, HandlerResult, Mint},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable,
    contract_utils::js_imports::Transaction,
    state::{KvState, Token},
    utils::is_op,
};

fn get_token_id(prefix: Option<String>, base_id: Option<String>) -> String {
    let base_id = base_id.unwrap_or_else(Transaction::id);
    prefix.map_or(base_id.clone(), |prefix| format!("{}-{}", prefix, base_id))
}

#[async_trait(?Send)]
impl AsyncActionable for Mint {
    async fn action(self, caller: String, state: Parameters) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        if !is_op(&caller).await {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        let token_id = get_token_id(self.prefix, self.base_id);

        token_id.chars().all(|c| c.is_alphanumeric() || c == '-');

        let default_token = KvState::settings().default_token().get().await;
        let ticker_nonce = KvState::ticker_nonce().get().await;

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

        KvState::ticker_nonce().map(|nonce| nonce + 1).await;

        Ok(HandlerResult::Write(state))
    }
}
