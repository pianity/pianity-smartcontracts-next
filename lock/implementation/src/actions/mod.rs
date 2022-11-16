use async_trait::async_trait;

use warp_lock::{action::ActionResult, state::State};

pub mod batch;
pub mod configure;
pub mod evolve;
pub mod transfer_locked;
pub mod unlock;

pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use transfer_locked::*;
pub use unlock::*;

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
