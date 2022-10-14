use async_trait::async_trait;

use warp_packs::{action::ActionResult, state::State};

pub mod batch;
pub mod configure;
pub mod evolve;
pub mod mint_pack;
pub mod open_pack;

pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use mint_pack::*;
pub use open_pack::*;

pub trait Actionable {
    fn action(self, caller: String, state: State) -> ActionResult;
}

#[async_trait(?Send)]
pub trait AsyncActionable {
    async fn action(self, caller: String, state: State) -> ActionResult;
}
