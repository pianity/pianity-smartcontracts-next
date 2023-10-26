use std::collections::HashMap;

use async_recursion::async_recursion;

use warp_scarcity::{
    action::{Action, ActionResult, Configure},
    error::ContractError,
    state::{InitialState, Parameters},
};

use crate::{
    actions::{self, Actionable, AsyncActionable},
    contract_utils::{
        foreign_call::ForeignContractCaller,
        js_imports::{log, Block, Contract, SmartWeave, Transaction},
    },
    state::{AttachedRoyalties, Settings, State},
    utils::{is_op, is_super_op},
};

pub async fn init(init_state: &InitialState) {
    let init_state = State {
        settings: Settings {
            paused: init_state.settings.paused,
            super_operators: init_state.settings.super_operators.clone(),
            operators: init_state.settings.operators.clone(),
            erc1155: init_state.settings.erc1155.clone(),
            custodian: init_state.settings.custodian.clone(),
        },
        all_attached_royalties: HashMap::from_iter(init_state.attached_royalties.iter().map(
            |(id, ar)| {
                (
                    id.clone(),
                    AttachedRoyalties {
                        base_id: ar.base_id.clone(),
                        royalties: ar.royalties.clone(),
                        rate: ar.rate,
                    },
                )
            },
        )),
    };

    State::init(&init_state).await
}

#[async_recursion(?Send)]
pub async fn handle(
    state: Parameters,
    action: Action,
    foreign_caller: &mut ForeignContractCaller,
) -> ActionResult {
    if let Some(init_state) = state.initial_state.as_ref() {
        init(init_state).await;
    }

    let direct_caller = SmartWeave::caller();

    // if state.settings.paused
    //     && std::mem::discriminant(&action)
    //         != std::mem::discriminant(&Action::Configure(Configure::default()))
    // {
    //     return Err(ContractError::ContractIsPaused);
    // }

    if State::settings().paused().get().await
        && std::mem::discriminant(&action)
            != std::mem::discriminant(&Action::Configure(Configure::default()))
    {
        return Err(ContractError::ContractIsPaused);
    }

    // NOTE: Currently, only Pianity is allowed to transfer NFTs
    if !is_op(&direct_caller).await && !is_super_op(&direct_caller).await {
        return Err(ContractError::UnauthorizedAddress(direct_caller));
    }

    match action {
        Action::AttachRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::EditAttachedRoyalties(action) => {
            action.action(direct_caller, state, foreign_caller).await
        }
        Action::Transfer(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Configure(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Evolve(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::Batch(action) => action.action(direct_caller, state, foreign_caller).await,
        Action::MintNft(action) => action.action(direct_caller, state, foreign_caller).await,
    }
}
