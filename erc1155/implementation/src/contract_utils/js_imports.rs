/////////////////////////////////////////////////////
/////////////// DO NOT MODIFY THIS FILE /////////////
/////////////////////////////////////////////////////

use async_trait::async_trait;
use kv_storage::KvStorage;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type Block;

    #[wasm_bindgen(static_method_of = Block, js_name = indep_hash)]
    pub fn indep_hash() -> String;

    #[wasm_bindgen(static_method_of = Block, js_name = height)]
    pub fn height() -> i32;

    #[wasm_bindgen(static_method_of = Block, js_name = timestamp)]
    pub fn timestamp() -> i32;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type Contract;

    #[wasm_bindgen(static_method_of = Contract, js_name = contractId)]
    pub fn id() -> String;

    #[wasm_bindgen(static_method_of = Contract, js_name = contractOwner)]
    pub fn owner() -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type Transaction;

    #[wasm_bindgen(static_method_of = Transaction, js_name = id)]
    pub fn id() -> String;

    #[wasm_bindgen(static_method_of = Transaction, js_name = owner)]
    pub fn owner() -> String;

    #[wasm_bindgen(static_method_of = Transaction, js_name = target)]
    pub fn target() -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type SmartWeave;

    #[wasm_bindgen(static_method_of = SmartWeave, js_name = readContractState)]
    pub async fn read_contract_state(contract_id: &str) -> JsValue;

    #[wasm_bindgen(static_method_of = SmartWeave, js_name = viewContractState)]
    pub async fn view_contract_state(contract_id: &str) -> JsValue;

    #[wasm_bindgen(static_method_of = SmartWeave, js_name = write)]
    pub async fn write(contract_id: &str, input: JsValue) -> JsValue;

    #[wasm_bindgen(static_method_of = SmartWeave, js_name = refreshState)]
    pub async fn refresh_state();

    #[wasm_bindgen(static_method_of = SmartWeave, js_name = caller)]
    pub fn caller() -> String;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type Vrf;

    #[wasm_bindgen(static_method_of = Vrf, js_name = value)]
    pub fn value() -> String;

    #[wasm_bindgen(static_method_of = Vrf, js_name = randomInt)]
    pub fn randomInt(max_value: i32) -> i32;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type KvJs;

    #[wasm_bindgen(static_method_of = KvJs)]
    pub async fn put(key: &str, value: JsValue);

    #[wasm_bindgen(static_method_of = KvJs)]
    pub async fn get(key: &str) -> JsValue;

    #[wasm_bindgen(static_method_of = KvJs)]
    pub async fn del(key: &str);
}

pub struct Kv;

#[async_trait(?Send)]
impl KvStorage for Kv {
    async fn put<T: Serialize>(key: &str, value: &T) {
        KvJs::put(key, JsValue::from_serde(value).unwrap()).await;
    }

    async fn get<T: DeserializeOwned>(key: &str) -> T {
        KvJs::get(key).await.into_serde().unwrap()
    }

    async fn del(key: &str) {
        KvJs::del(key).await;
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
