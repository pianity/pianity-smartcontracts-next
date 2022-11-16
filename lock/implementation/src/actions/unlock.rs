use std::{collections::HashMap, ops::Range};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State},
};

use warp_lock::{
    action::{ActionResult, HandlerResult, TransferLocked, Unlock},
    error::ContractError,
    state::{LockedBalance, State, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    actions::{Actionable, AsyncActionable},
    contract_utils::{
        foreign_call::ForeignContractCaller,
        js_imports::{Block, Contract},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

#[async_trait(?Send)]
impl AsyncActionable for Unlock {
    async fn action(
        self,
        _caller: String,
        mut state: State,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let lock_account = Contract::id();

        let current_block = Block::height() as u32;

        // Extracts the expired vaults from the state, mutating it in place.
        // NOTE: Would be much cleaner using `drain_filter` but it's not stable yet.
        // https://github.com/rust-lang/rust/issues/43244
        let expired_vault: HashMap<String, Vec<LockedBalance>> = state
            .vault
            .iter_mut()
            .map(|(owner, balances)| {
                let mut expired_balances = Vec::new();

                let mut i = 0;
                while i < balances.len() {
                    if balances[i].at + balances[i].duration <= current_block {
                        let expired_balance = balances.remove(i);
                        expired_balances.push(expired_balance);
                    } else {
                        i += 1;
                    }
                }

                (owner.to_string(), expired_balances)
            })
            .collect();

        let transfers: Vec<Erc1155Action::Action> = expired_vault
            .iter()
            .flat_map(|(owner, locked_balances)| {
                locked_balances.iter().map(|locked_balance| {
                    Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
                        from: Some(lock_account.clone()),
                        to: owner.clone(),
                        token_id: locked_balance.token_id.clone(),
                        qty: locked_balance.qty,
                    })
                })
            })
            .collect();

        if transfers.len() > 0 {
            foreign_caller
                .write::<Erc1155ContractError, Erc1155Action::Action>(
                    &state.settings.erc1155,
                    Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: transfers }),
                )
                .await
                .or_else(|err| Err(ContractError::Erc1155Error(err)))?;
        }

        Ok(HandlerResult::Write(state))
    }
}
