use async_trait::async_trait;

use warp_erc1155::action::{ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::Parameters as StateLegacy;

use crate::{
    actions::{approval::is_approved_for_all_internal, AsyncActionable},
    state::{Balance, State},
    utils::is_op,
};

#[async_trait(?Send)]
impl AsyncActionable for Transfer {
    async fn action(self, caller: String, state: StateLegacy) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        let from = if let Some(from) = self.from {
            from
        } else {
            caller.clone()
        };

        if !is_approved_for_all_internal(&caller, &from).await
            || (!State::settings().allow_free_transfer().get().await && !is_op(&caller).await)
        {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        if from == self.target {
            return Err(ContractError::TransferFromAndToCannotBeEqual);
        }

        let token_id = self
            .token_id
            .unwrap_or(State::settings().default_token().get().await);

        let token = State::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?;

        let from_balance = token
            .balances(&from)
            .peek()
            .await
            .unwrap_or(Balance::new(0));

        if from_balance.value < self.qty.value {
            return Err(ContractError::OwnerBalanceNotEnough(from));
        }

        let from_new_balance = Balance::new(from_balance.value - self.qty.value);

        if from_new_balance == Balance::new(0) {
            token.delete_balances(&from).await;
        } else {
            token.balances(&from).set(&from_new_balance).await;
        }

        token
            .balances(&self.target)
            .init(Balance::new(0))
            .await
            .map(|target_balance| Balance::new(target_balance.value + self.qty.value))
            .await;

        let target_balance = token.balances(&self.target).peek().await;
        let from_balance = token.balances(&from).peek().await;

        Ok(HandlerResult::None(state))
    }
}
