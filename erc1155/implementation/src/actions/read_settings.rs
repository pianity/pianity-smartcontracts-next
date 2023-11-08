use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, BalanceOf, HandlerResult, ReadResponse, ReadSettings},
    error::ContractError,
    state::{Balance as ActionBalance, Parameters, Settings},
};

use crate::{actions::AsyncActionable, contract_utils::js_imports::log};

use crate::state::{Balance, KvState};

#[async_trait(?Send)]
impl AsyncActionable for ReadSettings {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        log(&format!(
            "oooooooooooooo {:?}",
            KvState::settings().proxies().get().await,
        ));

        Ok(HandlerResult::Read(
            state,
            ReadResponse::ReadSettings(Settings {
                default_token: KvState::settings().default_token().get().await,
                paused: KvState::settings().paused().get().await,
                can_evolve: KvState::settings().can_evolve().get().await,
                super_operators: KvState::settings().super_operators().get().await,
                operators: KvState::settings().operators().get().await,
                proxies: KvState::settings().proxies().get().await,
                allow_free_transfer: KvState::settings().allow_free_transfer().get().await,
            }),
        ))
    }
}
