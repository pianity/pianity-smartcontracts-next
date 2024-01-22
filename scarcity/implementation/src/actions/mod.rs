use async_trait::async_trait;

use warp_scarcity::{action::ActionResult, state::Parameters};

pub mod attach_royalties;
pub mod batch;
pub mod configure;
pub mod edit_attached_royalties;
pub mod evolve;
pub mod get_all_royalties;
pub mod get_royalties;
pub mod initialize;
pub mod mint_nft;
pub mod remove_attached_royalties;
pub mod transfer;

pub use attach_royalties::*;











use crate::contract_utils::foreign_call::ForeignContractCaller;

pub trait Actionable {
    fn action(self, caller: String, state: Parameters) -> ActionResult;
}

#[async_trait(?Send)]
pub trait AsyncActionable {
    async fn action(
        self,
        caller: String,
        state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult;
}
