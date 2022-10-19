use async_trait::async_trait;

use warp_shuffle::{action::ActionResult, state::State};

pub mod batch;
pub mod configure;
pub mod evolve;
pub mod mint_shuffle;
pub mod open_shuffle;

pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use mint_shuffle::*;
pub use open_shuffle::*;

pub trait Actionable {
    fn action(self, caller: String, state: State) -> ActionResult;
}

#[async_trait(?Send)]
pub trait AsyncActionable {
    async fn action(self, caller: String, state: State) -> ActionResult;
}
