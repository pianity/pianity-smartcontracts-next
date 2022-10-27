use warp_erc1155::action::{ActionResult, Configure, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::{
    actions::Actionable,
    utils::{is_op, is_super_op},
};

impl Actionable for Configure {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        let is_super_op = is_super_op(&state, &caller);
        let is_op = is_op(&state, &caller);

        if self.super_operators.is_some() && !is_super_op
            || self.operators.is_some() && !is_super_op && (!is_super_op && is_op)
        {
            return Err(ContractError::UnauthorizedConfiguration);
        }

        if let Some(super_operators) = self.super_operators {
            state.settings.super_operators = super_operators;
        }

        if let Some(operators) = self.operators {
            state.settings.operators = operators;
        }

        if let Some(proxies) = self.proxies {
            state.settings.proxies = proxies;
        }

        return Ok(HandlerResult::Write(state));
    }
}
