use async_trait::async_trait;

use warp_erc1155::action::{ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::Parameters as StateLegacy;

use crate::contract_utils::js_imports::log;
use crate::{
    state::{Balance, KvState},
    utils::is_op,
};

use super::{is_approved_for_all_internal, AsyncActionable};

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
            || (!KvState::settings().allow_free_transfer().get().await && !is_op(&from).await)
        {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        if from != caller
            && !KvState::approvals(&from)
                .peek()
                .approves(&caller)
                .await
                .unwrap_or(false)
        {
            return Err(ContractError::UnauthorizedTransfer(from));
        }

        if from == self.target {
            return Err(ContractError::TransferFromAndToCannotBeEqual);
        }

        let token_id = self
            .token_id
            .unwrap_or(KvState::settings().default_token().get().await);

        let token = KvState::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?;

        let from_balance = token
            .balances(&from)
            .peek()
            .await
            .unwrap_or(Balance::new(0));

        if from_balance.value < self.qty.value {
            log(&format!(
                "BALANCE TOO LOW transfer: {} -> {} of {}; owner has {}",
                from, self.target, self.qty.value, from_balance.value
            ));
            return Err(ContractError::OwnerBalanceNotEnough(from));
        }

        log(&format!(
            "transfer: {} -> {} of {}",
            from, self.target, self.qty.value
        ));

        let from_new_balance = Balance::new(from_balance.value - self.qty.value);

        if from_new_balance == Balance::new(0) {
            log(&format!("[erc1155] removing balance for {}", from));
            token.delete_balances(&from).await;
            // token.balances(&from).set(&Balance::new(0)).await;
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

        log(&format!(
            "updated balances:\n\
            \t-> {}: {:?}\n\
            \t-> {}: {:?}",
            from, from_balance, self.target, target_balance
        ));

        Ok(HandlerResult::None(state))
    }
}
