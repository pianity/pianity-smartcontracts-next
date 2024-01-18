use async_trait::async_trait;
use warp_erc1155::action::{ActionResult, Burn, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::Parameters;

use crate::state::{Balance, KvState};
use crate::{actions::AsyncActionable, utils::is_op};

#[async_trait(?Send)]
impl AsyncActionable for Burn {
    async fn action(self, caller: String, state: Parameters) -> ActionResult {
        if !is_op(&caller).await {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        let owner = if let Some(owner) = self.owner {
            owner.clone()
        } else {
            caller
        };

        let token_id = self
            .token_id
            .unwrap_or(KvState::settings().default_token().get().await);

        let token = KvState::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?;

        let balance = token
            .balances(&owner)
            .peek()
            .await
            .unwrap_or(Balance::new(0))
            .value;

        if balance < self.qty.value {
            return Err(ContractError::OwnerBalanceNotEnough(owner));
        } else if balance == self.qty.value {
            token.delete_balances(&owner).await;

            if token.count_balances().await == 0 {
                KvState::delete_tokens(&token_id).await;
            }
        } else {
            token
                .balances(&owner)
                .map(|balance| Balance::new(balance.value - self.qty.value))
                .await;
        }

        Ok(HandlerResult::None(state))
    }
}
