use warp_shuffle::{
    action::{ActionResult, Configure, HandlerResult},
    error::ContractError,
    state::State,
};

use crate::{
    actions::Actionable,
    contract_utils::foreign_call::ForeignContractCaller,
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

        if let Some(paused) = self.paused {
            state.settings.paused = paused;
        }

        if let Some(erc1155) = self.erc1155 {
            state.settings.erc1155 = erc1155;
        }

        if let Some(custodian) = self.custodian {
            state.settings.custodian = custodian;
        }

        if let Some(boost_token) = self.boost_token {
            state.settings.boost_token = boost_token;
        }

        if let Some(boost_price_modifier) = self.boost_price_modifier {
            state.settings.boost_price_modifier = boost_price_modifier;
        }

        if let Some(boost_cap) = self.boost_cap {
            state.settings.boost_cap = boost_cap;
        }

        Ok(HandlerResult::Write(state))
    }
}
