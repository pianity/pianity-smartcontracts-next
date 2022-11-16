use async_trait::async_trait;

use warp_scarcity::{action::ActionResult, state::State};

pub mod attach_royalties;
pub mod batch;
pub mod configure;
pub mod evolve;
pub mod mint_nft;
pub mod transfer;

pub use attach_royalties::*;
pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use mint_nft::*;
pub use transfer::*;

use crate::contract_utils::foreign_call::ForeignContractCaller;

pub trait Actionable {
    fn action(self, caller: String, state: State) -> ActionResult;
}

#[async_trait(?Send)]
pub trait AsyncActionable {
    async fn action(
        self,
        caller: String,
        state: State,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult;
}
