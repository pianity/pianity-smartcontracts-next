use async_trait::async_trait;

use warp_lock::{action::ActionResult, state::Parameters};

pub mod batch;
pub mod configure;
pub mod evolve;
pub mod get_all_vaults;
pub mod get_vault;
pub mod initialize;
pub mod transfer_locked;
pub mod unlock;

pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use get_all_vaults::*;
pub use get_vault::*;
pub use initialize::*;
pub use transfer_locked::*;
pub use unlock::*;

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
