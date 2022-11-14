use async_trait::async_trait;
use warp_shuffle::{
    action::{Action, ActionResult, Batch, HandlerResult, ReadResponse},
    error::ContractError,
    state::State,
};

use crate::{contract::handle, contract_utils::foreign_call::ForeignContractCaller};

use super::AsyncActionable;

#[async_trait(?Send)]
impl AsyncActionable for Batch {
    async fn action(
        self,
        _caller: String,
        mut state: State,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let mut results: Vec<ReadResponse> = Vec::new();

        let mut read_mode = false;
        let mut write_mode = false;

        for action in self.actions {
            if let Action::Batch(_) = action {
                return Err(ContractError::ForbiddenNestedBatch);
            }

            state = match handle(state, action, foreign_caller).await? {
                HandlerResult::Write(state) => {
                    write_mode = true;

                    if read_mode {
                        return Err(ContractError::CannotMixReadAndWrite);
                    }

                    state
                }
                HandlerResult::Read(state, response) => {
                    read_mode = true;

                    if write_mode {
                        return Err(ContractError::CannotMixReadAndWrite);
                    }

                    results.push(response);
                    state
                }
            }
        }

        if read_mode {
            Ok(HandlerResult::Read(state, ReadResponse::Batch(results)))
        } else if write_mode {
            Ok(HandlerResult::Write(state))
        } else {
            Err(ContractError::EmptyBatch)
        }
    }
}
