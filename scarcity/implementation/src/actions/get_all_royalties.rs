use async_trait::async_trait;
use warp_scarcity::{
    action::{ActionResult, GetAllRoyalties, HandlerResult, ReadResponse},
    state::{AttachedRoyalties, Parameters},
};

use crate::{
    actions::AsyncActionable, contract_utils::foreign_call::ForeignContractCaller, state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for GetAllRoyalties {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        _foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let all_royalties = State::list_all_attached_royalties()
            .await
            .into_iter()
            .map(|(base_id, royalties)| {
                (
                    base_id,
                    AttachedRoyalties {
                        base_id: royalties.base_id,
                        royalties: royalties.royalties,
                        rate: royalties.rate,
                    },
                )
            })
            .collect();

        Ok(HandlerResult::Read(
            state,
            ReadResponse::GetAllRoyalties(all_royalties),
        ))
    }
}
