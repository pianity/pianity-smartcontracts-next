use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State},
};

use warp_lock::{
    action::{ActionResult, HandlerResult, TransferLocked},
    error::ContractError,
    state::{LockedBalance, State, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::contract_utils::js_imports::log;
use crate::{
    actions::{Actionable, AsyncActionable},
    contract_utils::{
        foreign_call::write_foreign_contract,
        js_imports::{Block, Contract},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

#[async_trait(?Send)]
impl AsyncActionable for TransferLocked {
    async fn action(self, caller: String, mut state: State) -> ActionResult {
        let lock_account = Contract::id();

        let transfer = Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
            from: Some(caller.clone()),
            to: lock_account.clone(),
            token_id: self.token_id.clone(),
            qty: self.qty,
        });

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            transfer,
        )
        .await
        .or_else(|err| Err(ContractError::Erc1155Error(err)))?;

        let current_block = Block::height();

        state.vault.entry(self.to).or_default().push(LockedBalance {
            token_id: self.token_id,
            qty: self.qty,
            from: caller,
            at: current_block as u32,
            duration: self.duration,
        });

        Ok(HandlerResult::Write(state))
    }
}
