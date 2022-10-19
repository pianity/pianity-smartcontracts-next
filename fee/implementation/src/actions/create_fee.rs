use async_trait::async_trait;
use serde::Deserialize;

use warp_fee::{
    action::{ActionResult, CreateFee, HandlerResult},
    error::ContractError,
    state::{Fees, Nft, State, UNIT},
};

use crate::contract_utils::foreign_call::read_foreign_contract_state;
use crate::{
    actions::{Actionable, AsyncActionable},
    utils::splited_nft_id,
};

use warp_erc1155::state::{State as Erc1155State, Token as Erc1155Token};

pub fn create_fee_internal(create_fee: &CreateFee, state: &mut State) -> Result<(), ContractError> {
    if create_fee.rate > UNIT {
        return Err(ContractError::InvalidRate);
    }

    // Check that the sum of all fees is equal to UNIT
    let fees_sum = create_fee
        .fees
        .iter()
        .map(|(_, fee)| *fee)
        .reduce(|sum, fee| sum + fee)
        .unwrap_or(0);

    if fees_sum != UNIT {
        return Err(ContractError::InvalidFee);
    }

    state.nfts.insert(
        create_fee.nft_base_id.clone(),
        Nft {
            base_id: create_fee.nft_base_id.clone(),
            fees: create_fee.fees.clone(),
            rate: create_fee.rate,
        },
    );

    Ok(())
}

#[async_trait(?Send)]
impl AsyncActionable for CreateFee {
    async fn action(self, _caller: String, mut state: State) -> ActionResult {
        if state.nfts.contains_key(&self.nft_base_id) {
            return Err(ContractError::TokenAlreadyExists(self.nft_base_id));
        }

        let erc1155: Erc1155State = read_foreign_contract_state(&state.settings.erc1155).await;

        erc1155
            .tokens
            .iter()
            // find all existing tokens attached to `nft_base_id`
            .filter(|(id, _)| splited_nft_id(id).is_some())
            // find whether at least one of these tokens isn't an nft
            .find(|(_, token)| {
                token
                    .balances
                    .iter()
                    .map(|(_, balance)| balance.value)
                    .reduce(|sum, balance| sum + balance)
                    .unwrap_or(0)
                    != 1
            })
            .map_or(Ok(()), |(id, _)| {
                Err(ContractError::TokenIsNotAnNFT(id.to_string()))
            })?;

        create_fee_internal(&self, &mut state)?;

        Ok(HandlerResult::Write(state))
    }
}
