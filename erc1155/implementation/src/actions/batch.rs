use async_trait::async_trait;
use warp_erc1155::{
    action::{Action, ActionResult, Batch, HandlerResult, ReadResponse},
    error::ContractError,
    state::State,
};

use crate::contract::handle;

use super::AsyncActionable;

#[async_trait(?Send)]
impl AsyncActionable for Batch {
    async fn action(self, caller: String, mut state: State) -> ActionResult {
        let mut results: Vec<ReadResponse> = Vec::new();

        let mut read_mode = false;
        let mut write_mode = false;
        let mut none_mode = false;

        for action in self.actions {
            if let Action::Batch(_) = action {
                return Err(ContractError::ForbiddenNestedBatch);
            }

            state = match handle(state, action).await? {
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashMap;
//     use warp_erc1155::state::{BalancePrecision, Balances, Token};
//
//     #[test]
//     fn batch_test() {
//         fn to_balance(res: &ReadResponse) -> BalancePrecision {
//             if let ReadResponse::Balance { balance, .. } = res {
//                 return *balance;
//             } else {
//                 panic!()
//             }
//         }
//
//         let state = State {
//             tokens: HashMap::from([(
//                 "a".to_string(),
//                 Token {
//                     ticker: "a".to_owned(),
//                     balances: HashMap::new(),
//                 },
//             )]),
//             ..Default::default()
//         };
//
//         let args = Batch {
//             actions: vec![Action::BalanceOf {
//                 token_id: "a".to_string(),
//                 target: "a".to_string(),
//             }],
//         };
//
//         let expected_responses = vec![1];
//         let result: HandlerResult = tokio_test::block_on(super::batch(state, args)).unwrap();
//
//         if let HandlerResult::Read(_, ReadResponse::Batch(responses)) = &result {
//             let test = responses
//                 .iter()
//                 .zip(expected_responses)
//                 .filter(|(res, b)| to_balance(res) == *b)
//                 .count();
//
//             assert!(test > 0);
//         } else {
//             panic!()
//         }
//     }
// }
