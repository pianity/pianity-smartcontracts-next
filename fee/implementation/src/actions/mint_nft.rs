use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::{Balance, State as Erc1155State},
};

use warp_fee::{
    action::{ActionResult, CreateFee, HandlerResult, MintNft},
    error::ContractError,
    state::{Fees, State, Token, UNIT},
};
use wasm_bindgen::UnwrapThrowExt;

use crate::contract_utils::{
    foreign_call::read_foreign_contract_state,
    js_imports::{log, Transaction},
};
use crate::{
    actions::{Actionable, AsyncActionable},
    contract_utils::foreign_call::write_foreign_contract,
};

use super::create_fee_internal;

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

async fn get_token_owner(erc1155: &str, token_id: &str) -> Option<String> {
    let state = read_foreign_contract_state::<Erc1155State>(&erc1155.to_string()).await;

    let token = state.tokens.get(token_id)?;

    let owner = token.balances.iter().next().unwrap_throw();

    Some(owner.0.clone())
}

#[async_trait(?Send)]
impl AsyncActionable for MintNft {
    async fn action(self, _caller: String, mut state: State) -> ActionResult {
        let mut mints = Vec::new();

        let (scarcity_name, editions_count) = match self.scarcity {
            warp_fee::action::NftScarcity::Unique => ("UNIQUE", 1),
            warp_fee::action::NftScarcity::Legendary => ("LEGENDARY", 10),
            warp_fee::action::NftScarcity::Epic => ("EPIC", 100),
            warp_fee::action::NftScarcity::Rare => ("RARE", 1000),
        };

        for edition in 0..editions_count {
            let prefix = format!("{}-{}", scarcity_name, edition + 1);
            let token_id = format!(
                "{}-{}",
                prefix,
                self.ticker.clone().unwrap_or_else(Transaction::id)
            );

            let create_fee = CreateFee {
                token_id,
                rate: self.rate,
                fees: self.fees.clone(),
            };

            create_fee_internal(&create_fee, &mut state)?;

            mints.push(Erc1155Action::Action::Mint(Erc1155Action::Mint {
                ticker: None,
                prefix: Some(prefix),
                qty: Balance::new(1),
            }));
        }

        let transaction_batch =
            Erc1155Action::Action::Batch(Erc1155Action::Batch { actions: mints });

        write_foreign_contract::<InternalWriteResult, Erc1155ContractError, Erc1155Action::Action>(
            &state.settings.erc1155,
            transaction_batch,
        )
        .await
        .or_else(|err| Err(ContractError::Erc1155Error(err)))?;

        Ok(HandlerResult::Write(state))
    }
}
