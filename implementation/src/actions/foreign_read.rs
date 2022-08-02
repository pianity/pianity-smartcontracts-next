use warp_erc1155::action::{ActionResult, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::contract_utils::foreign_call::read_foreign_contract_state;
use crate::contract_utils::js_imports::log;

pub async fn foreign_read(mut state: State, contract_tx_id: String) -> ActionResult {
    if contract_tx_id == "bad_contract" {
        Err(ContractError::IDontLikeThisContract)
    } else {
        let foreign_contract_state: State = read_foreign_contract_state(&contract_tx_id).await;

        // // Some dummy logic - just for the sake of the integration test
        // if foreign_contract_state.ticker == "FOREIGN_PST" {
        //     log("Adding to tokens");
        //     for val in state.balances.values_mut() {
        //         *val += 1000;
        //     }
        // }

        Ok(HandlerResult::Write(state))
    }
}
