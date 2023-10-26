use std::collections::HashMap;

use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult, Configure, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{InitialState, Parameters};

use crate::contract_utils::js_imports::{log, KvJs};
use crate::state::{Approvals, Balance, Settings, Token};
use crate::{
    actions::{Actionable, AsyncActionable, *},
    contract_utils::js_imports::{SmartWeave, Transaction},
    state::KvState,
};

pub async fn init(init_state: &InitialState) {
    let state = &KvState {
        ticker_nonce: init_state.ticker_nonce,
        tokens: HashMap::from_iter(init_state.tokens.iter().map(|(id, token)| {
            (
                id.clone(),
                Token {
                    tx_id: token.tx_id.clone(),
                    ticker: token.ticker.clone(),
                    balances: HashMap::from_iter(token.balances.iter().map(
                        |(address, balance)| {
                            log(&format!("{}: {}: {}", id, address, balance.value));
                            (address.clone(), Balance::new(balance.value))
                        },
                    )),
                },
            )
        })),
        approvals: HashMap::from_iter(init_state.approvals.iter().map(|(address, approvals)| {
            (
                address.clone(),
                Approvals {
                    approves: HashMap::from_iter(
                        approvals
                            .approves
                            .iter()
                            .map(|(address, approved)| (address.clone(), approved.clone())),
                    ),
                },
            )
        })),
        settings: Settings {
            default_token: init_state.settings.default_token.clone(),
            paused: init_state.settings.paused,
            can_evolve: init_state.settings.can_evolve,
            super_operators: init_state.settings.super_operators.clone(),
            operators: init_state.settings.operators.clone(),
            proxies: init_state.settings.proxies.clone(),
            allow_free_transfer: init_state.settings.allow_free_transfer,
        },
    };

    KvState::init(state).await
}

#[async_recursion(?Send)]
pub async fn handle(mut state: Parameters, action: Action) -> ActionResult {
    if let Some(init_state) = state.initial_state.as_ref() {
        init(&init_state).await;
        state.initial_state = None;
    }

    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    // if state.settings.paused
    //     && std::mem::discriminant(&action)
    //         != std::mem::discriminant(&Action::Configure(Configure::default()))
    // {
    //     return Err(ContractError::ContractIsPaused);
    // }

    let effective_caller = if KvState::settings()
        .proxies()
        .get()
        .await
        .contains(&direct_caller)
    {
        original_caller
    } else {
        direct_caller
    };

    match action {
        Action::Transfer(action) => action.action(effective_caller, state).await,
        Action::BalanceOf(action) => action.action(effective_caller, state).await,
        Action::Configure(action) => action.action(effective_caller, state).await,
        Action::Evolve(action) => action.action(effective_caller, state).await,
        Action::SetApprovalForAll(action) => action.action(effective_caller, state).await,
        Action::IsApprovedForAll(action) => action.action(effective_caller, state).await,
        Action::Mint(action) => action.action(effective_caller, state).await,
        Action::Burn(action) => action.action(effective_caller, state).await,
        Action::Batch(action) => action.action(effective_caller, state).await,
    }
}
