use async_trait::async_trait;
use warp_erc1155::{
    action::{Action, ActionResult, Batch, HandlerResult, ReadResponse},
    error::ContractError,
    state::Parameters,
};

use crate::{actions::AsyncActionable, contract::execute_action};

#[async_trait(?Send)]
impl AsyncActionable for Batch {
    async fn action(self, caller: String, mut state: Parameters) -> ActionResult {
        let mut results: Vec<ReadResponse> = Vec::new();

        let mut read_mode = false;
        let mut write_mode = false;
        let mut none_mode = false;

        for action in self.actions {
            if let Action::Batch(_) = action {
                return Err(ContractError::ForbiddenNestedBatch);
            }

            state = match execute_action(Box::new(action), caller.clone(), state).await? {
                HandlerResult::Write(state) => {
                    write_mode = true;

                    if read_mode {
                        return Err(ContractError::CannotMixeReadAndWrite);
                    }

                    state
                }
                HandlerResult::Read(state, response) => {
                    read_mode = true;

                    if write_mode {
                        return Err(ContractError::CannotMixeReadAndWrite);
                    }

                    results.push(response);
                    state
                }
                HandlerResult::None(state) => {
                    none_mode = true;

                    state
                }
            }
        }

        if read_mode {
            Ok(HandlerResult::Read(state, ReadResponse::Batch(results)))
        } else if write_mode {
            Ok(HandlerResult::Write(state))
        } else if none_mode {
            Ok(HandlerResult::None(state))
        } else {
            Err(ContractError::EmptyBatch)
        }
    }
}
