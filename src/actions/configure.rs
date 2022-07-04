use crate::action::{ActionResult, ConfigureArgs};
use crate::contract_utils::handler_result::HandlerResult;
use crate::contract_utils::js_imports::Transaction;
use crate::error::ContractError;
use crate::state::State;

pub fn configure(mut state: State, args: ConfigureArgs) -> ActionResult {
    let caller = Transaction::owner();

    if args.super_owner.is_some() && caller != state.settings.super_owner
        || args.owners.is_some() && caller != state.settings.super_owner
        || args.authorized_addresses.is_some()
            && (caller != state.settings.super_owner && state.settings.owners.contains(&caller))
    {
        return Err(ContractError::UnauthorizedConfiguration);
    }

    if let Some(super_owner) = args.super_owner {
        state.settings.super_owner = super_owner;
    }

    if let Some(owners) = args.owners {
        state.settings.owners = owners;
    }

    if let Some(authorized_addresses) = args.authorized_addresses {
        state.settings.authorized_addresses = authorized_addresses;
    }

    return Ok(HandlerResult::Write(state));
}
