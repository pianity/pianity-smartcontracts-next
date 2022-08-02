use serde::{Deserialize, Serialize};
use warp_erc1155::action::{ActionResult, HandlerResult};
use warp_erc1155::state::State;

use crate::contract_utils::foreign_call::write_foreign_contract;
use crate::contract_utils::js_imports::log;

#[derive(Serialize)]
struct Input {
    function: String,
    qty: u64,
    target: String,
}

#[derive(Deserialize)]
struct Result {
    state: State,
    #[serde(rename = "type")]
    result_type: String,
}

pub async fn foreign_write(
    state: State,
    contract_tx_id: String,
    qty: u64,
    target: String,
) -> ActionResult {
    let result: Result = write_foreign_contract(
        &contract_tx_id,
        Input {
            function: "transfer".to_string(),
            qty,
            target,
        },
    )
    .await;

    // log(("Write done! ".to_owned() + &result.state.ticker).as_str());
    // log(("Result type ".to_owned() + &result.result_type).as_str());

    Ok(HandlerResult::Write(state))
}
