use std::collections::HashMap;

use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, HandlerResult, Initialize},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::{AttachedRoyalties, Settings},
};

use crate::state::State;

#[async_trait(?Send)]
impl AsyncActionable for Initialize {
    async fn action(
        self,
        _caller: String,
        mut parameters: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if let Some(init_state) = parameters.initial_state {
            let init_state = State {
                settings: Settings {
                    paused: init_state.settings.paused,
                    super_operators: init_state.settings.super_operators.clone(),
                    operators: init_state.settings.operators.clone(),
                    erc1155: init_state.settings.erc1155.clone(),
                    custodian: init_state.settings.custodian.clone(),
                },
                all_attached_royalties: HashMap::from_iter(
                    init_state.attached_royalties.iter().map(|(id, ar)| {
                        (
                            id.clone(),
                            AttachedRoyalties {
                                base_id: ar.base_id.clone(),
                                royalties: ar.royalties.clone(),
                                rate: ar.rate,
                            },
                        )
                    }),
                ),
            };

            State::init(&init_state).await;

            parameters.initial_state = None;

            Ok(HandlerResult::Write(parameters))
        } else {
            Err(ContractError::ContractAlreadyInitialized)
        }
    }
}
