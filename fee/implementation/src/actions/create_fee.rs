use async_trait::async_trait;
use serde::Deserialize;

use warp_fee::{
    action::{ActionResult, CreateFee, HandlerResult},
    error::ContractError,
    state::{Fees, Nft, State, UNIT},
};

use crate::actions::{Actionable, AsyncActionable};
use crate::contract_utils::foreign_call::read_foreign_contract_state;

// use crate::contract_utils::js_imports::Transaction;
// use crate::error::ContractError;
// use crate::state::{Fees, State, Token};

use warp_erc1155::state::State as Erc1155State;

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
        create_fee.nft_id.clone(),
        Nft {
            id: create_fee.nft_id.clone(),
            fees: create_fee.fees.clone(),
            rate: create_fee.rate,
        },
    );

    Ok(())
}

#[async_trait(?Send)]
impl AsyncActionable for CreateFee {
    async fn action(self, _caller: String, mut state: State) -> ActionResult {
        if state.nfts.contains_key(&self.nft_id) {
            return Err(ContractError::TokenAlreadyExists(self.nft_id));
        }

        let erc1155: Erc1155State = read_foreign_contract_state(&state.settings.erc1155).await;

        // Make sure the token is an NFT
        if let Some(token) = erc1155.tokens.get(&self.nft_id) {
            if token
                .balances
                .iter()
                .map(|(_, balance)| balance.value)
                .reduce(|sum, balance| sum + balance)
                .unwrap_or(0)
                != 1
            {
                return Err(ContractError::TokenIsNotAnNFT(self.nft_id));
            }
        } else {
            return Err(ContractError::TokenNotFound(self.nft_id));
        }

        if !erc1155.tokens.contains_key(&self.nft_id) {
            return Err(ContractError::TokenDoesNotExist(self.nft_id));
        }

        create_fee_internal(&self, &mut state)?;

        Ok(HandlerResult::Write(state))
    }
}
