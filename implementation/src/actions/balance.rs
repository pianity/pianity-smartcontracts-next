use warp_erc1155::{
    action::{ActionResult, HandlerResult, ReadResponse},
    error::ContractError,
    state::State,
};

pub fn balance_of(state: State, caller: String, token_id: String, target: String) -> ActionResult {
    let balance = {
        let token = match state.tokens.get(&token_id) {
            Some(token) => token,
            None => return Err(ContractError::TokenNotFound(token_id)),
        };

        match token.balances.get(&target) {
            Some(balance) => balance.clone().value,
            None => 0,
        }
    };

    Ok(HandlerResult::Read(
        state,
        ReadResponse::Balance { balance, target },
    ))
}
