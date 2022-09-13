use warp_erc1155::action::{ActionResult, Configure, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::contract_utils::js_imports::Transaction;

use super::Actionable;

impl Actionable for Configure {
    fn action(self, caller: String, mut state: State) -> ActionResult {
        if self.super_owner.is_some() && caller != state.settings.super_operator
            || self.owners.is_some() && caller != state.settings.super_operator
            || self.transfer_proxies.is_some()
                && (caller != state.settings.super_operator
                    && state.settings.operators.contains(&caller))
        {
            return Err(ContractError::UnauthorizedConfiguration);
        }

        if let Some(super_owner) = self.super_owner {
            state.settings.super_operator = super_owner;
        }

        if let Some(owners) = self.owners {
            state.settings.operators = owners;
        }

        if let Some(transfer_proxies) = self.transfer_proxies {
            state.settings.transfer_proxies = transfer_proxies;
        }

        return Ok(HandlerResult::Write(state));
    }
}
