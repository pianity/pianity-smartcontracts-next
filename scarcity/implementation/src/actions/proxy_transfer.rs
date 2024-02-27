use async_trait::async_trait;

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, BalancePrecision},
};

use warp_scarcity::{
    action::{ActionResult, HandlerResult, ProxyTransfer},
    error::ContractError,
    state::{Parameters, UNIT},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::ForeignContractCaller,
    state::State,
    utils::{NftId, ShuffleId, TokenId},
};

#[async_trait(?Send)]
impl AsyncActionable for ProxyTransfer {
    async fn action(
        self,
        caller: String,
        state: Parameters,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        let base_id = match TokenId::from(self.token_id.as_ref()) {
            TokenId::Nft(NftId { base_id, .. }) => base_id,
            TokenId::Shuffle(ShuffleId { base_id }) => base_id,
            TokenId::Token(token_id) => token_id,
        };

        let has_attached_royalties = State::all_attached_royalties(&base_id)
            .peek()
            .await
            .is_some();

        if has_attached_royalties {
            return Err(ContractError::CantUseProxyTransferOnTokenWithRoyalties(
                base_id,
            ));
        }

        let transfer = Erc1155Action::Action::Transfer(Erc1155Action::Transfer {
            from: Some(self.from.unwrap_or(caller)),
            target: self.target,
            token_id: Some(self.token_id),
            qty: self.qty,
        });

        foreign_caller
            .write::<Erc1155ContractError, Erc1155Action::Action>(
                &State::settings().erc1155().get().await,
                transfer,
            )
            .await
            .map_err(ContractError::Erc1155Error)?;

        Ok(HandlerResult::Write(state))
    }
}
