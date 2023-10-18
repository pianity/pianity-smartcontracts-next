use async_recursion::async_recursion;

use warp_erc1155::action::{Action, ActionResult, Configure, HandlerResult};
use warp_erc1155::error::ContractError;
use warp_erc1155::state::State;

use crate::contract_utils::js_imports::{log, Kv, KvJs};
use crate::{
    actions::{Actionable, AsyncActionable, *},
    contract_utils::js_imports::{SmartWeave, Transaction},
    state::KvState,
};

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

    let value = serde_wasm_bindgen::to_value::<[u32]>(&[]).unwrap();

    log(&format!("CONTRACT putting \"{:?}\"", value));
    KvJs::put(".settings.ticker_nonce", value).await.unwrap();

    log(&format!(
        "CONTRACT get: \"{:?}\"",
        KvJs::get(".settings.ticker_nonce").await.unwrap()
    ));

    // KvState::init(&KvState::default()).await;
    //
    // KvState::settings().test().set(&vec![123, 456]).await;
    //
    // KvState::settings().proxies().get().await;
    //
    // KvState::settings().proxies().get().await;
    //
    // KvState::settings().test().get().await;
    //
    // KvState::settings().ticker_nonce().get().await;
    //
    // KvState::settings().default_token().get().await;
    //
    // KvState::settings().paused().get().await;

    KvState::settings().allow_free_transfer().get().await;

    let effective_caller = if KvState::settings()
        .proxies()
        .get()
        .await
        .contains(&direct_caller)
    {
        original_caller
    } else {
        direct_caller
    };

    // let effective_caller = if state.settings.proxies.contains(&direct_caller) {
    //     original_caller
    // } else {
    //     direct_caller
    // };

    match action {
        Action::Transfer(action) => action.action(String::new(), state).await,
        Action::BalanceOf(action) => action.action(effective_caller, state).await,
        Action::Configure(action) => action.action(effective_caller, state).await,
        Action::Evolve(action) => action.action(effective_caller, state),
        Action::SetApprovalForAll(action) => action.action(effective_caller, state).await,
        Action::IsApprovedForAll(action) => action.action(effective_caller, state).await,
        Action::Mint(action) => action.action(effective_caller, state).await,
        Action::Burn(action) => action.action(effective_caller, state).await,
        Action::Batch(action) => action.action(effective_caller, state).await,
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
