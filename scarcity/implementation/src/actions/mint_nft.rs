use async_trait::async_trait;

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::Balance,
};

use warp_scarcity::{
    action::{ActionResult, AttachRoyalties, HandlerResult, MintNft},
    error::ContractError,
    state::Parameters,
};

use crate::{
    actions::{attach_royalties::attach_royalties_internal, AsyncActionable},
    contract_utils::{foreign_call::ForeignContractCaller, js_imports::Transaction},
    state::State,
};

#[async_trait(?Send)]
impl AsyncActionable for MintNft {
    async fn action(
        self,
        _caller: String,
        state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let mut mints = Vec::new();

        let (scarcity_name, editions_count): (String, u32) =
            (self.scarcity.to_string(), (&self.scarcity).into());

        attach_royalties_internal(&AttachRoyalties {
            base_id: self.base_id.clone().unwrap_or_else(Transaction::id),
            rate: self.rate,
            royalties: self.royalties.clone(),
        })
        .await?;

        for edition in 0..editions_count {
            let prefix = format!("{}-{}", edition + 1, scarcity_name);

            mints.push(Erc1155Action::Action::Mint(Erc1155Action::Mint {
                base_id: self.base_id.clone(),
                prefix: Some(prefix),
                qty: Balance::new(1),
            }));
        }

        let transaction_batch =
            Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: mints });

        foreign_caller
            .write::<Erc1155ContractError, Erc1155Action::Action>(
                &State::settings().erc1155().get().await,
                transaction_batch,
            )
            .await
            .map_err(ContractError::Erc1155Error)?;

        Ok(HandlerResult::None(state))
    }
}
