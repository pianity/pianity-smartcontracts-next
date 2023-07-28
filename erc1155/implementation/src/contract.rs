use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult, Configure, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::actions::{Actionable, *};
use crate::contract_utils::js_imports::{SmartWeave, Transaction};

#[async_recursion(?Send)]
pub async fn handle(state: State, action: Action) -> ActionResult {
    let original_caller = Transaction::owner();
    let direct_caller = SmartWeave::caller();

    // if state.settings.paused
    //     && std::mem::discriminant(&action)
    //         != std::mem::discriminant(&Action::Configure(Configure::default()))
    // {
    //     return Err(ContractError::ContractIsPaused);
    // }
    //
    // let effective_caller = if state.settings.proxies.contains(&direct_caller) {
    //     original_caller
    // } else {
    //     direct_caller
    // };

    match action {
        Action::Transfer(action) => action.action(String::new(), state).await,
        _ => unimplemented!()
        // Action::BalanceOf(action) => action.action(effective_caller, state),
        // Action::Configure(action) => action.action(effective_caller, state),
        // Action::Evolve(action) => action.action(effective_caller, state),
        // Action::SetApprovalForAll(action) => action.action(effective_caller, state),
        // Action::IsApprovedForAll(action) => action.action(effective_caller, state),
        // Action::Mint(action) => action.action(effective_caller, state),
        // Action::Burn(action) => action.action(effective_caller, state),
        // Action::Batch(action) => action.action(effective_caller, state).await,
    }
}

// #[cfg(test)]
// mod tests {
//     use kv_macro::kv_storage;
//     use wasm_bindgen::JsValue;
//
//     // struct Kv;
//     // impl Kv {
//     //     async fn put(key: &str, value: JsValue) {
//     //         println!("put {} {:?}", key, value);
//     //     }
//     //
//     //     async fn get(key: &str) -> JsValue {
//     //         println!("get {}", key);
//     //
//     //         JsValue::from("")
//     //     }
//     // }
//     //
//     // struct DummyValue;
//     // impl Displayk
//
//     #[kv_storage]
//     struct State {
//         name: String,
//     }
//
//     #[test]
//     fn test_macro() {
//         let test: String = String::new();
//         test.to_string();
//
//         let name_node = StorageItemname(".name".to_string());
//         let value = tokio_test::block_on(name_node.value());
//
//         println!("value: {:?}", value);
//
//         // let state = hey();
//     }
// }
