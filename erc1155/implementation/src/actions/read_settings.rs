use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, HandlerResult, ReadResponse, ReadSettings},
    state::{Parameters, Settings},
};

use crate::actions::AsyncActionable;

use crate::state::KvState;

#[async_trait(?Send)]
impl AsyncActionable for ReadSettings {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        Ok(HandlerResult::Read(
            state,
            ReadResponse::ReadSettings(Settings {
                default_token: KvState::settings().default_token().get().await,
                paused: KvState::settings().paused().get().await,
                super_operators: KvState::settings().super_operators().get().await,
                operators: KvState::settings().operators().get().await,
                proxies: KvState::settings().proxies().get().await,
                allow_free_transfer: KvState::settings().allow_free_transfer().get().await,
            }),
        ))
    }
}
