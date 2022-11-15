use async_trait::async_trait;

use warp_scarcity::{
    action::{ActionResult, AttachFee, HandlerResult},
    error::ContractError,
    state::{AttachedFee, State, UNIT},
};

use crate::{
    actions::AsyncActionable,
    contract_utils::foreign_call::{ForeignContractCaller, ForeignContractState},
    utils::parse_token_id,
};

pub fn attach_fee_internal(attach_fee: &AttachFee, state: &mut State) -> Result<(), ContractError> {
    if attach_fee.rate > UNIT {
        return Err(ContractError::InvalidRate);
    }

    // Check that the sum of all fees is equal to UNIT
    let fees_sum = attach_fee
        .fees
        .iter()
        .map(|(_, fee)| *fee)
        .reduce(|sum, fee| sum + fee)
        .unwrap_or(0);

    if fees_sum != UNIT {
        return Err(ContractError::InvalidFee);
    }

    state.attached_fees.insert(
        attach_fee.base_id.clone(),
        AttachedFee {
            base_id: attach_fee.base_id.clone(),
            fees: attach_fee.fees.clone(),
            rate: attach_fee.rate,
        },
    );

    Ok(())
}

#[async_trait(?Send)]
impl AsyncActionable for AttachFee {
    async fn action(
        self,
        _caller: String,
        mut state: State,
        foreign_caller: &mut ForeignContractCaller,
    ) -> ActionResult {
        if state.attached_fees.contains_key(&self.base_id) {
            return Err(ContractError::TokenAlreadyExists(self.base_id));
        }

        // let erc1155 = match foreign_caller
        //     .read(&state.settings.erc1155.to_string())
        //     .await
        //     .map_err(|_err| ContractError::Erc1155ReadFailed)?
        // {
        //     ForeignContractState::Erc1155(state) => state,
        // };

        // erc1155
        //     .tokens
        //     .iter()
        //     // find all existing shuffles and nfts attached to `nft_base_id`
        //     .filter(|(id, _)| {
        //         // splitted_nft_id(id).map_or(false, |(_, _, base_id)| base_id == self.base_id)
        //         parse_token_id(id).map_or(false, |(_, base_id)| base_id == self.base_id)
        //     })
        //     // // find whether at least one of these tokens isn't an nft
        //     // .find(|(_, token)| {
        //     //     token
        //     //         .balances
        //     //         .iter()
        //     //         .map(|(_, balance)| balance.value)
        //     //         .reduce(|sum, balance| sum + balance)
        //     //         .unwrap_or(0)
        //     //         != 1
        //     // })
        //     // .map_or(Ok(()), |(id, _)| {
        //     //     Err(ContractError::TokenIsNotAnNFT(id.to_string()))
        //     // })?;

        attach_fee_internal(&self, &mut state)?;

        Ok(HandlerResult::Write(state))
    }
}
