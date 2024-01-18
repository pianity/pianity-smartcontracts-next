use std::collections::HashMap;

use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, HandlerResult, Initialize},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable,
    state::{Approvals, Settings, Token},
};

use crate::state::{Balance, KvState};

#[async_trait(?Send)]
impl AsyncActionable for Initialize {
    async fn action(self, _caller: String, mut parameters: Parameters) -> ActionResult {
        if let Some(init_state) = parameters.initial_state {
            let state = &KvState {
                ticker_nonce: init_state.ticker_nonce,
                tokens: HashMap::from_iter(init_state.tokens.iter().map(|(id, token)| {
                    (
                        id.clone(),
                        Token {
                            tx_id: token.tx_id.clone(),
                            ticker: token.ticker.clone(),
                            balances: HashMap::from_iter(token.balances.iter().map(
                                |(address, balance)| (address.clone(), Balance::new(balance.value)),
                            )),
                        },
                    )
                })),
                approvals: HashMap::from_iter(init_state.approvals.iter().map(
                    |(address, approvals)| {
                        (
                            address.clone(),
                            Approvals {
                                approves: HashMap::from_iter(approvals.approves.iter().map(
                                    |(address, approved)| (address.clone(), approved.clone()),
                                )),
                            },
                        )
                    },
                )),
                settings: Settings {
                    default_token: init_state.settings.default_token.clone(),
                    paused: init_state.settings.paused,
                    super_operators: init_state.settings.super_operators.clone(),
                    operators: init_state.settings.operators.clone(),
                    proxies: init_state.settings.proxies.clone(),
                    allow_free_transfer: init_state.settings.allow_free_transfer,
                },
            };

            KvState::init(state).await;

            parameters.initial_state = None;

            Ok(HandlerResult::Write(parameters))
        } else {
            Err(ContractError::ContractAlreadyInitialized)
        }
    }
}
