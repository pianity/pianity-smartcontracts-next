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

        if !is_op
            || (self.super_operators.is_some() && !is_super_op)
            || (self.operators.is_some() && !is_super_op)
            || (self.can_evolve.is_some() && !is_super_op)
        {
            return Err(ContractError::UnauthorizedConfiguration);
        }

        if let Some(super_operators) = self.super_operators {
            state.settings.super_operators = super_operators;
        }

        if let Some(operators) = self.operators {
            state.settings.operators = operators;
        }

        if let Some(can_evolve) = self.can_evolve {
            state.settings.can_evolve = can_evolve;
        }

        if let Some(proxies) = self.proxies {
            state.settings.proxies = proxies;
        }

        if let Some(paused) = self.paused {
            state.settings.paused = paused;
        }

        if let Some(allow_free_transfer) = self.allow_free_transfer {
            state.settings.allow_free_transfer = allow_free_transfer;
        }

        return Ok(HandlerResult::Write(state));
    }
}
