use warp_scarcity::{
    action::{ActionResult, AttachRoyalties, EditAttachedRoyalties, HandlerResult},
    error::ContractError,
    state::State,
};

use crate::actions::{attach_royalties_internal, Actionable};

impl Actionable for EditAttachedRoyalties {
    fn action(self, _caller: String, mut state: State) -> ActionResult {
        if !state.all_attached_royalties.contains_key(&self.base_id) {
            return Err(ContractError::TokenNotFound(self.base_id));
        }

        attach_royalties_internal(
            &AttachRoyalties {
                base_id: self.base_id,
                royalties: self.royalties,
                rate: self.rate,
            },
            &mut state,
        )?;

        Ok(HandlerResult::Write(state))
    }
}
