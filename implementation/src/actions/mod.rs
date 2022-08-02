use warp_erc1155::{action::ActionResult, state::State};

pub mod approval;
pub mod balance;
pub mod batch;
pub mod configure;
pub mod evolve;
pub mod foreign_read;
pub mod foreign_write;
pub mod mint;
pub mod transfer;

pub use approval::*;
pub use balance::*;
pub use batch::*;
pub use configure::*;
pub use evolve::*;
pub use foreign_read::*;
pub use foreign_write::*;
pub use mint::*;
pub use transfer::*;

pub trait Actionable {
    fn action(self, caller: String, state: State) -> ActionResult;
}
