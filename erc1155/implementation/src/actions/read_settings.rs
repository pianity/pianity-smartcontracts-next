use async_trait::async_trait;
use warp_erc1155::{
    action::{ActionResult, HandlerResult, ReadResponse, ReadSettings},
    state::{Parameters, Settings},
};

use crate::actions::AsyncActionable;

use crate::state::State;

#[async_trait(?Send)]
impl AsyncActionable for ReadSettings {
    async fn action(self, _caller: String, state: Parameters) -> ActionResult {
        Ok(HandlerResult::Read(
            state,
            ReadResponse::ReadSettings(Settings {
                default_token: State::settings().default_token().get().await,
                paused: State::settings().paused().get().await,
                super_operators: State::settings().super_operators().get().await,
                operators: State::settings().operators().get().await,
                proxies: State::settings().proxies().get().await,
                allow_free_transfer: State::settings().allow_free_transfer().get().await,
            }),
        ))
    }
}
