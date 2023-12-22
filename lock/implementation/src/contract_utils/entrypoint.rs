/////////////////////////////////////////////////////
/////////////// DO NOT MODIFY THIS FILE /////////////
/////////////////////////////////////////////////////

use std::cell::RefCell;

use serde::Serialize;

use wasm_bindgen::prelude::*;

use warp_lock::{
    action::{Action, HandlerResult, ReadResponse},
    error::ContractError,
    state::Parameters,
};

use crate::{contract, contract_utils::foreign_call::ForeignContractCaller};

use super::js_imports::log;

/*
Note: in order do optimize communication between host and the WASM module,
we're storing the state inside the WASM module (for the time of state evaluation).
This allows to reduce the overhead of passing the state back and forth
between the host and module with each contract interaction.
In case of bigger states this overhead can be huge.
Same approach has been implemented for the AssemblyScript version.

So the flow (from the SDK perspective) is:
1. SDK calls exported WASM module function "initState" (with lastly cached state or initial state,
if cache is empty) - which initializes the state in the WASM module.
2. SDK calls "handle" function for each of the interaction.
If given interaction was modifying the state - it is updated inside the WASM module
- but not returned to host.
3. Whenever SDK needs to know the current state (eg. in order to perform
caching or to simply get its value after evaluating all of the interactions)
- it calls WASM's module "currentState" function.

The handle function by default does not return the new state -
it only updates it in the WASM module.
The handle function returns a value only in case of error
or calling a "view" function.

In the future this might also allow to enhance the inner-contracts communication
- e.g. if the execution network will store the state of the contracts - as the WASM contract module memory
- it would allow to read other contract's state "directly" from WASM module memory.
*/

// inspired by https://github.com/dfinity/examples/blob/master/rust/basic_dao/src/basic_dao/src/lib.rs#L13
thread_local! {
    static STATE: RefCell<Parameters> = RefCell::default();
}

#[wasm_bindgen()]
pub async fn handle(interaction: JsValue) -> Option<JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();

    let action = serde_wasm_bindgen::from_value::<Action>(interaction);

    if action.is_err() {
        let error = Err::<HandlerResult, _>(ContractError::RuntimeError(
            "Error while parsing input".to_string(),
        ));

        return Some(error.serialize(&serializer).unwrap());
    }

    let state = STATE.with(|service| service.borrow().clone());
    let mut foreign_caller = ForeignContractCaller::new();
    let result = contract::handle(state, action.unwrap(), &mut foreign_caller).await;

    match result {
        Ok(HandlerResult::Write(state)) => {
            STATE.with(|service| service.replace(state));
            None
        }
        Ok(HandlerResult::Read(_, response)) => Some(
            Ok::<ReadResponse, ContractError>(response)
                .serialize(&serializer)
                .unwrap(),
        ),
        Ok(HandlerResult::None(_)) => None,
        error @ Err(_) => Some(error.serialize(&serializer).unwrap()),
    }
}

#[wasm_bindgen(js_name = initState)]
pub fn init_state(state: &JsValue) {
    let state_parsed: Parameters = state.into_serde().unwrap();

    STATE.with(|service| service.replace(state_parsed));
}

#[wasm_bindgen(js_name = currentState)]
pub fn current_state() -> JsValue {
    // not sure if that's deterministic - which is very important for the execution network.
    // TODO: perf - according to docs:
    // "This is unlikely to be super speedy so it's not recommended for large payload"
    // - we should minimize calls to from_serde
    let current_state = STATE.with(|service| service.borrow().clone());
    JsValue::from_serde(&current_state).unwrap()
}

#[wasm_bindgen()]
pub fn version() -> i32 {
    return 1;
}

// Workaround for now to simplify type reading without as/loader or wasm-bindgen
// 1 = assemblyscript
// 2 = rust
// 3 = go
// 4 = swift
// 5 = c
#[wasm_bindgen]
pub fn lang() -> i32 {
    return 2;
}
