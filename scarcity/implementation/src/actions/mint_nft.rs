use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use warp_erc1155::{
    action::{self as Erc1155Action},
    error::ContractError as Erc1155ContractError,
    state::Balance,
};

use warp_scarcity::{
    action::{ActionResult, CreateFee, HandlerResult, MintNft},
    error::ContractError,
    state::State,
};

use crate::contract_utils::js_imports::{log, Transaction};
use crate::{actions::AsyncActionable, contract_utils::foreign_call::write_foreign_contract};

use super::create_fee_internal;

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
}

#[async_trait(?Send)]
impl AsyncActionable for MintNft {
    async fn action(self, _caller: String, mut state: State) -> ActionResult {
        let mut mints = Vec::new();

        let (scarcity_name, editions_count) = match self.scarcity {
            warp_scarcity::action::NftScarcity::Unique => ("UNIQUE", 1),
            warp_scarcity::action::NftScarcity::Legendary => ("LEGENDARY", 10),
            warp_scarcity::action::NftScarcity::Epic => ("EPIC", 100),
            warp_scarcity::action::NftScarcity::Rare => ("RARE", 1000),
        };

        let nft_base_id = self.ticker.clone().unwrap_or_else(Transaction::id);

        create_fee_internal(
            &CreateFee {
                nft_base_id: nft_base_id.clone(),
                rate: self.rate,
                fees: self.fees.clone(),
            },
            &mut state,
        )?;

        for edition in 0..editions_count {
            let prefix = format!("{}-{}", edition + 1, scarcity_name);

            mints.push(Erc1155Action::Action::Mint(Erc1155Action::Mint {
                ticker: Some(nft_base_id.clone()),
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
