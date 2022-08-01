use std::collections::HashMap;

use crate::action::{ActionResult, MintArgs};
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::{Balance, Balances, State, Token};
use crate::utils::is_op;

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

pub fn mint(mut state: State, caller: String, args: MintArgs) -> ActionResult {
    if args.qty.value == 0 {
        return Err(ContractError::TransferAmountMustBeHigherThanZero);
    }

    if !(is_op(&state, &caller)) {
        return Err(ContractError::UnauthorizedAddress(caller));
    }

    let token_id = get_token_id(args.prefix, args.ticker);

    token_id.chars().all(|c| c.is_alphanumeric() || c == '-');

    if state.tokens.get(&token_id).is_some() {
        return Err(ContractError::TokenAlreadyExists);
    }

    let token = Token {
        // TODO: What should `ticker` be? Is it necessary?
        ticker: token_id.clone(),
        balances: HashMap::from([("".to_string(), Balance::new(args.qty.value))]),
    };

    state.tokens.insert(token_id, token);

    Ok(HandlerResult::Write(state))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        action::{HandlerResult, MintArgs},
        state::{Balance, Settings, State, Token},
    };

    use super::mint;

    fn as_write(result: HandlerResult) -> State {
        if let HandlerResult::Write(state) = result {
            state
        } else {
            panic!("Result isn't a write");
        }
    }

    #[test]
    fn mint_test() {
        let state = State {
            settings: Settings {
                ..Default::default()
            },
            // tokens: HashMap::from([(
            //     "a".to_string(),
            //     Token {
            //         ticker: "a".to_string(),
            //         balances: HashMap::from([("".to_string(), 1)]),
            //     },
            // )]),
            ..Default::default()
        };

        let state = as_write(
            mint(
                state,
                "".to_string(),
                MintArgs {
                    ticker: None,
                    prefix: None,
                    qty: Balance::new(1),
                },
            )
            .unwrap(),
        );
    }
}
