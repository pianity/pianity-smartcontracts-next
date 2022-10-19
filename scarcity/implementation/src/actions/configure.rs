use warp_scarcity::{
    action::{ActionResult, Configure, HandlerResult},
    error::ContractError,
    state::State,
};

use crate::actions::Actionable;

impl Actionable for Configure {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.super_operator.is_some() && caller != state.settings.super_operator
            || self.operators.is_some()
                && caller != state.settings.super_operator
                && (caller != state.settings.super_operator
                    && state.settings.operators.contains(&caller))
        {
            return Err(ContractError::UnauthorizedConfiguration);
        }

        if let Some(super_operator) = self.super_operator {
            state.settings.super_operator = super_operator;
        }

        if let Some(operators) = self.operators {
            state.settings.operators = operators;
        }

        return Ok(HandlerResult::Write(state));
    }
}
