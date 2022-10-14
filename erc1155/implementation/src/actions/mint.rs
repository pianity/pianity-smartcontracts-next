use warp_erc1155::action::{ActionResult, HandlerResult, Mint};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{State, Token};

use crate::contract_utils::js_imports::Transaction;
use crate::utils::is_op;

use super::Actionable;

fn get_token_id(prefix: Option<String>, ticker: Option<String>) -> String {
    let ticker = ticker.unwrap_or_else(Transaction::id);
    prefix.map_or(ticker.clone(), |prefix| format!("{}-{}", prefix, ticker))
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

        state
            .tokens
            .entry(token_id.clone())
            .or_insert(Token {
                ticker: token_id.clone(),
                ..Default::default()
            })
            .balances
            .entry(caller.clone())
            .or_default()
            .value += self.qty.value;

        Ok(HandlerResult::Write(state))
    }
}
