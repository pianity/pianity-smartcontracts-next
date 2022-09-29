use std::collections::HashMap;

use warp_erc1155::action::{ActionResult, HandlerResult, Mint};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{Balance, State, Token};

use crate::contract_utils::js_imports::Transaction;
use crate::utils::is_op;

use super::Actionable;

fn get_token_id(prefix: Option<String>, ticker: Option<String>) -> String {
    let tx_id = Transaction::id();

    let ticker = ticker.unwrap_or(tx_id);

    let token_id = if let Some(prefix) = prefix {
        format!("{}-{}", prefix, ticker)
    } else {
        ticker
    };

    token_id
}

impl Actionable for Mint {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        if !(is_op(&state, &caller)) {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        let token_id = get_token_id(self.prefix, self.ticker);

        token_id.chars().all(|c| c.is_alphanumeric() || c == '-');

        if state.tokens.get(&token_id).is_some() {
            return Err(ContractError::TokenAlreadyExists);
        }

        let token = Token {
            ticker: token_id.clone(),
            balances: HashMap::from([(caller.to_string(), Balance::new(self.qty.value))]),
        };

        state.tokens.insert(token_id, token);

        Ok(HandlerResult::Write(state))
    }
}
