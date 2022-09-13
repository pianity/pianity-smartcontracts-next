use async_trait::async_trait;
use warp_erc1155::{action::ActionResult, state::State};

pub mod approval;
pub mod balance;
pub mod batch;
pub mod configure;
pub mod evolve;
pub mod mint;
pub mod transfer;

pub use approval::*;
pub use balance::*;
pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use mint::*;
pub use transfer::*;

use crate::contract_utils::js_imports::log;

pub trait Actionable {
    fn action(self, caller: String, state: State) -> ActionResult;
}

#[async_trait(?Send)]
pub trait AsyncActionable {
    async fn action(self, caller: String, state: State) -> ActionResult;
}
