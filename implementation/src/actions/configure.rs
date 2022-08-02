use warp_erc1155::action::{ActionResult, ConfigureArgs, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::contract_utils::js_imports::Transaction;

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
