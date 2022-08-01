use crate::action::{ActionResult, ConfigureArgs};
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::State;

pub fn configure(mut state: State, caller: String, args: ConfigureArgs) -> ActionResult {
    if args.super_owner.is_some() && caller != state.settings.super_operator
        || args.owners.is_some() && caller != state.settings.super_operator
        || args.authorized_addresses.is_some()
            && (caller != state.settings.super_operator
                && state.settings.operators.contains(&caller))
    {
        return Err(ContractError::UnauthorizedConfiguration);
    }

    if let Some(super_owner) = args.super_owner {
        state.settings.super_operator = super_owner;
    }

    if let Some(owners) = args.owners {
        state.settings.operators = owners;
    }

    if let Some(authorized_addresses) = args.authorized_addresses {
        state.settings.authorized_addresses = authorized_addresses;
    }

    return Ok(HandlerResult::Write(state));
}
