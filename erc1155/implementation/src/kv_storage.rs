use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsValue;

use kv_storage::{kv_storage_macro, StorageItem};
use warp_erc1155::{
    action::Mint,
    state::{Balance, Info, Settings},
};

use crate::contract_utils::js_imports::log;

// pub mod Storage {
//     use kv_storage::StorageItem;
//
//     use crate::contract_utils::js_imports::Kv;
//
//     pub struct name;
//
//     impl StorageItem<Kv> for name {
//         type Value<'b> = String;
//
//         const PATH: &'static str = "name";
//     }
// }

// .tokens
//     .TOKEN_ID
//         .ticker -> string
//         .balances
//             .ADDRESS -> string
// .settings
//     .paused -> bool
//     .operators -> string[]

pub struct Settings2 {
    paused: bool,
    operators: Vec<String>,
}

// // #[kv_storage_macro]
// pub struct Storage {
//     name: String,
//     #[kv_storage_macro(subkey)]
//     settings: Settings2,
// }

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    // use crate::contract_utils::js_imports::Kv;
    use kv_storage::{kv_storage_macro, KvStorage, StorageItem};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json;
    use wasm_bindgen::JsValue;

    struct Kv;

    #[async_trait(?Send)]
    impl KvStorage for Kv {
        async fn put<T: Serialize>(key: &str, value: &T) {
            println!(
                "{}",
                &format!("put: {} = {:?}", key, serde_json::to_string(value).unwrap())
            );
        }

        async fn get<T: DeserializeOwned>(key: &str) -> T {
            println!(
                "{}",
                &format!("put: {} = {:?}", key, serde_json::to_string(key).unwrap())
            );

            serde_json::from_str("\"bonjour\"").unwrap()

            // println!("{}", &format!("get: {}", key));
            // JsValue::from_serde(&"test").unwrap()
        }

        async fn del(key: &str) {
            println!("{}", &format!("del: {}", key));
        }
    }

    // fn test<T: KvStorage>() {
    //     T::put("foo", JsValue::from("bar"));
    // }

    #[derive(Serialize, Deserialize, Debug)]
    enum Bob {
        Foo,
        Bar,
    }

    // #[kv_storage_macro(subkey)]
    #[kv_storage_macro(Kv)]
    struct State {
        name: String,
        something: Bob,
    }

    #[test]
    fn test_macro() {
        tokio_test::block_on(State::name().set_value(String::from("hello")));
        let name = tokio_test::block_on(State::name().value());
        println!("TEST {:?}", name);
        // let name =
        // println!("TEST {:?}", tokio_test::block_on(state().value()));
        // foo();
    }
}

// #[async_trait(?Send)]
// trait GetRange {
//     const PATH: &'static str;
//     async fn get_range(&self) -> Vec<JsValue>;
// }
//
// #[async_trait(?Send)]
// trait GetChild {
//     // const PATH: &'static str;
//     async fn get_child(&self, child: &str) -> JsValue;
// }

// #[async_trait(?Send)]
// trait StorageItem<T: KvStorage> {
//     type Value<'b>: for<'a> Deserialize<'a> + Serialize + 'b;
//     const PATH: &'static str;
//
//     async fn set_value<'b>(&self, value: Self::Value<'b>) {
//         T::put(&self.key(), JsValue::from_serde(&value).unwrap()).await;
//     }
//
//     async fn value<'b>(&self) -> Self::Value<'b> {
//         T::get(&self.key()).await.into_serde().unwrap()
//     }
//
//     async fn update<'b, F: FnOnce(&mut Self::Value<'b>)>(&self, update_fn: F) {
//         let mut value = self.value().await;
//         update_fn(&mut value);
//         self.set_value(value).await;
//     }
//
//     fn key(&self) -> String;
// }

// trait StorageMap {
//     // type Value: for<'a> Deserialize<'a> + Serialize;
//     type Value: StorageLeaf;
//
//     // async fn set(&self, key: &str, value: Self::Value) {
//     //     Kv::put(
//     //         &format!("{}.{}", self.rootKey(), key),
//     //         JsValue::from_serde(&value).unwrap(),
//     //     )
//     //     .await
//     // }
//
//     fn get(&self, key: &str) -> Self::Value {
//         Self::Value::new(format!("{}.{}", self.root_key(), key))
//         // Kv::get(&format!("{}.{}", self.rootKey(), key))
//         //     .await
//         //     .into_serde()
//         //     .unwrap()
//     }
//
//     fn root_key(&self) -> String;
// }
//
// trait StorageLeaf {
//     fn new(item_key: String) -> Self;
// }
//
// struct BalanceItem(String);
// impl StorageLeaf for BalanceItem {
//     fn new(item_key: String) -> Self {
//         BalanceItem(item_key)
//     }
// }
// impl StorageItem for BalanceItem {
//     type Value = Balance;
//
//     fn key(&self) -> String {
//         self.0.clone()
//     }
// }
//
// struct BalancesMap(String);
// impl StorageMap for BalancesMap {
//     type Value = BalanceItem;
//
//     fn root_key(&self) -> String {
//         self.0.clone()
//     }
// }
//
// struct TokenNode(String);
// impl TokenNode {
//     fn ticker(&self) -> TickerItem {
//         TickerItem(self.0.clone())
//     }
//     fn balances(&self) -> BalancesMap {
//         BalancesMap(self.0.clone())
//     }
// }
// impl StorageLeaf for TokenNode {
//     fn new(item_key: String) -> Self {
//         Self(item_key)
//     }
// }
//
// struct Root(String);
//
// impl Root {
//     fn tokens(&self) -> Tokens {
//         Tokens(self.0.clone())
//     }
// }
//
// struct TickerItem(String);
// impl StorageItem for TickerItem {
//     type Value = String;
//
//     fn key(&self) -> String {
//         format!("{}.ticker", self.0)
//     }
// }
//
// struct Tokens(String);
// impl StorageMap for Tokens {
//     type Value = TokenNode;
//
//     fn root_key(&self) -> String {
//         self.0.clone()
//     }
// }
//
// async fn test() {
//     let root = Root(".".to_string());
//     root.tokens()
//         .get("1")
//         .balances()
//         .get("address1")
//         .value()
//         .await;
// }
//
// // #[async_trait(?Send)]
// // impl GetChild for Tokens {
// //     async fn get_child(&self, child: &str) -> JsValue {
// //         // Kv::get(&format!("{}.{}", Self::PATH, child)).await
// //
// //         JsValue::from(0)
// //     }
// // }
// //
// // pub async fn create_token(default_token: &str, ticker_nounce: u32, token: Mint) {
// //     let ticker = format!("{}{}", state.default_token, state.ticker_nonce);
// //
// //     Kv::put(
// //         &format!("tokens.{}.ticker", token.base_id),
// //         JsValue::from_serde(&ticker).unwrap(),
// //     )
// //     .await;
// //
// //     Kv::put(
// //         &format!("tokens.{}.info", token.token_id),
// //         JsValue::from_serde(&Info {
// //             name: token.name,
// //             symbol: token.symbol,
// //             decimals: token.decimals,
// //         })
// //         .unwrap(),
// //     )
// //     .await;
// // }
//
// pub async fn get_token_balance(token_id: &str, address: &str) -> Option<Balance> {
//     Kv::get(&format!("tokens.{}.balances.{}", token_id, address))
//         .await
//         .into_serde()
//         .ok()
// }
//
// pub async fn set_token_balance(token_id: &str, address: &str, balance: Balance) {
//     Kv::put(
//         &format!("tokens.{}.balances.{}", token_id, address),
//         JsValue::from_serde(&balance).unwrap(),
//     )
//     .await
// }
//
// pub async fn get_token_ticker(token_id: &str) -> Option<Balance> {
//     Kv::get(&format!("tokens.{}.ticker", token_id))
//         .await
//         .into_serde()
//         .ok()
// }
//
// pub async fn get_token_tx_id(token_id: &str) -> Option<Balance> {
//     Kv::get(&format!("tokens.{}.txId", token_id))
//         .await
//         .into_serde()
//         .ok()
// }
//
// pub async fn get_approval(owner: &str, operator: &str) -> Option<bool> {
//     Kv::get(&format!("approvals.{}.{}", owner, operator))
//         .await
//         .into_serde()
//         .ok()
// }
//
// pub async fn get_settings() -> Settings {
//     Kv::get("settings").await.into_serde().unwrap()
// }
//
// pub async fn get_info() -> Info {
//     Kv::get("info").await.into_serde().unwrap()
// }
