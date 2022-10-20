use async_trait::async_trait;
use serde::Deserialize;

use crate::action::{ActionResult, Actionable, AsyncActionable, HandlerResult};
use crate::contract_utils::foreign_call::read_foreign_contract_state;
use crate::contract_utils::handler_result::HandlerResult::Write;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::{Fees, State, Token};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRead {
    token_id: String,
    fees: Fees,
}

#[async_trait(?Send)]
impl AsyncActionable for TestRead {
    async fn action(self, caller: String, mut state: State) -> ActionResult {
        // if state.tokens.contains_key(&self.token_id) {
        //     return Err(ContractError::TokenAlreadyExists(self.token_id));
        // }
        //
        // let erc1155: Erc1155State = read_foreign_contract_state(&state.settings.erc1155).await;
        //
        // if !erc1155.tokens.contains_key(&self.token_id) {
        //     return Err(ContractError::TokenDoesNotExist(self.token_id));
        // }
        //
        // state.tokens.insert(
        //     self.token_id.clone(),
        //     Token {
        //         id: self.token_id,
        //         fees: self.fees,
        //     },
        // );

        Ok(HandlerResult::Write(state))
    }
}
