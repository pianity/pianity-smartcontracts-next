use async_trait::async_trait;
use wasm_bindgen::JsValue;

use warp_erc1155::action::{ActionResult, HandlerResult, Transfer};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::{Info, State as StateLegacy, Token};

use crate::contract_utils::js_imports::{log, Kv};
use crate::state::{Balance, KvState};
use crate::utils::is_op;

// use super::approval::is_approved_for_all_internal;
use super::AsyncActionable;

// async fn is_approved(owner: &str, operator: &str) -> bool {
//     // match KvState::approvals(&owner).get().await {
//     //     Some(approves) => approves
//     //         .approvals(operator)
//     //         .get()
//     //         .await
//     //     None => false,
//     // }
//
//     if let Some(approves) = KvState::approvals(&owner).get().await {
//         if let Some(approves) = approves.approvals(operator).get().await {
//             approves.get().await
//         } else {
//             false
//         }
//     } else {
//         false
//     }
// }

#[async_trait(?Send)]
impl AsyncActionable for Transfer {
    async fn action(self, caller: String, mut state: StateLegacy) -> ActionResult {
        if self.qty.value == 0 {
            return Err(ContractError::TransferAmountMustBeHigherThanZero);
        }

        let from = if let Some(from) = self.from {
            from
        } else {
            caller.clone()
        };

        if KvState::settings().allow_free_transfer().get().await {
            return Err(ContractError::UnauthorizedAddress(caller));
        }

        // if !state.settings.allow_free_transfer && !is_op(&state, &caller) {
        //     return Err(ContractError::UnauthorizedAddress(caller));
        // }

        if from != caller
            && !KvState::approvals(&from)
                .peek()
                .approves(&caller)
                .await
                .unwrap_or(false)
        {
            return Err(ContractError::UnauthorizedTransfer(from));
        }

        if from == self.to {
            return Err(ContractError::TransferFromAndToCannotBeEqual);
        }

        let token_id = self
            .token_id
            .unwrap_or(KvState::settings().default_token().get().await);

        // let token = state
        //     .tokens
        //     .get_mut(token_id)
        //     .ok_or_else(|| ContractError::TokenNotFound(token_id.clone()))?;

        let token = KvState::tokens(&token_id)
            .ok_or(ContractError::TokenNotFound(token_id.clone()))
            .await?;
        // token.sdfgupbsdfg();

        // let token = {
        //     let token_id = self.token_id.as_ref().unwrap_or(&state.default_token);
        //     // KvState::tokens(token_id).
        // };

        // Checking if caller has enough funds
        // let from_balance = *token.balances.get(&from).unwrap_or(&Balance::new(0));

        let from_balance = token
            .balances(&from)
            .peek()
            .await
            .unwrap_or(Balance::new(0));

        if from_balance.value < self.qty.value {
            return Err(ContractError::OwnerBalanceNotEnough(from));
        }

        let from_new_balance = Balance::new(from_balance.value - self.qty.value);

        if from_new_balance.value > 0 {
            token.balances(&from).set(&from_new_balance).await;
        }

        // if from_new_balance.value == 0 {
        //     token.balances.remove(&from);
        // } else {
        //     // Update caller balance
        //     token.balances.insert(from, from_new_balance);
        // }

        let target_balance = token
            .balances(&self.to)
            .peek()
            .await
            .unwrap_or(Balance::new(0));
        token
            .balances(&self.to)
            .set(&Balance::new(target_balance.value + self.qty.value))
            .await;

        token
            .balances(&self.to)
            .init(Balance::new(0))
            .await
            .set(&Balance::new(target_balance.value + self.qty.value))
            .await;

        // // Update target balance
        // let target_balance = *token.balances.get(&self.to).unwrap_or(&Balance::new(0));
        // token
        //     .balances
        //     .insert(self.to, Balance::new(target_balance.value + self.qty.value));

        Ok(HandlerResult::None(state))
    }
}
