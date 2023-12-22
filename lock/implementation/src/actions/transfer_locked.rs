use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, Parameters as Erc1155State},
};

use warp_lock::{
    action::{ActionResult, HandlerResult, ReleaseMethod, TransferLocked},
    error::ContractError,
    state::{Cliff, Linear, LockedBalance, Parameters, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::{
    actions::{Actionable, AsyncActionable},
    contract_utils::js_imports::{Block, Contract},
};
use crate::{contract_utils::foreign_call::ForeignContractCaller, state::State};

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

#[async_trait(?Send)]
impl AsyncActionable for TransferLocked {
    async fn action(
        self,
        caller: String,
        state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let lock_account = Contract::id();

        let transfer = Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
            from: Some(caller.clone()),
            target: lock_account.clone(),
            token_id: Some(self.token_id.clone()),
            qty: self.qty,
        });

        foreign_caller
            .write::<Erc1155ContractError, Erc1155Action::Action>(
                &State::settings().erc1155().get().await,
                transfer,
            )
            .await
            .or_else(|err| Err(ContractError::Erc1155Error(err)))?;

        let current_block = Block::height();

        let locked_balance = match self.method {
            ReleaseMethod::Cliff => LockedBalance::Cliff(Cliff {
                token_id: self.token_id.clone(),
                qty: self.qty,
                from: caller.clone(),
                at: current_block as u32,
                duration: self.duration,
            }),
            ReleaseMethod::Linear => LockedBalance::Linear(Linear {
                token_id: self.token_id.clone(),
                qty: self.qty,
                from: caller.clone(),
                at: current_block as u32,
                duration: self.duration,
                unlocked: Balance::new(0),
            }),
        };

        State::vault(&self.target)
            .init_default()
            .await
            .map(|mut balances| {
                balances.push(locked_balance);
                balances
            })
            .await;

        Ok(HandlerResult::None(state))
    }
}
