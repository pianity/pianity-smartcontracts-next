/////////////////////////////////////////////////////
/////////////// DO NOT MODIFY THIS FILE /////////////
/////////////////////////////////////////////////////

use once_cell::sync::OnceCell;
// use parking_lot::Mutex;
use std::sync::Mutex;

use serde_json::Error;
use wasm_bindgen::prelude::*;

use crate::action::{Action, QueryResponseMsg};
use crate::contract;
use crate::contract_utils::handler_result::HandlerResult;
use crate::error::ContractError;
use crate::state::State;

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

static STATE: OnceCell<Mutex<State>> = OnceCell::new();

fn with_state<R>(data: Mutex<State>, f: impl FnOnce(&mut State) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}

#[wasm_bindgen()]
pub async fn handle(interaction: JsValue) -> Option<JsValue> {
    let result: Result<HandlerResult<QueryResponseMsg>, ContractError>;
    let action: Result<Action, Error> = interaction.into_serde();

    if action.is_err() {
        // cannot pass any data from action.error here - ends up with
        // "FnOnce called more than once" error from wasm-bindgen for
        // "foreign_call" testcase.
        result = Err(ContractError::RuntimeError(
            "Error while parsing input".to_string(),
        ));
    } else {
        // let state = &mut *STATE.get().unwrap().lock().unwrap();
        let state = STATE.get().unwrap().into_inner().unwrap();

        result = contract::handle(state, action.unwrap()).await;
    }

    if let Ok(HandlerResult::NewState) = result {
        None
    } else {
        Some(JsValue::from_serde(&result).unwrap())
    }
}

#[wasm_bindgen(js_name = initState)]
pub fn init_state(state: &JsValue) {
    let state_parsed: State = state.into_serde().unwrap();

    STATE.set(Mutex::new(state_parsed)).unwrap();
}

#[wasm_bindgen(js_name = currentState)]
pub fn current_state() -> JsValue {
    // not sure if that's deterministic - which is very important for the execution network.
    // TODO: perf - according to docs:
    // "This is unlikely to be super speedy so it's not recommended for large payload"
    // - we should minimize calls to from_serde
    let current_state = STATE.get().unwrap();
    JsValue::from_serde(current_state).unwrap()
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
