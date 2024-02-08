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

    #[wasm_bindgen(catch, static_method_of = KvJs, js_name = kvGet)]
    pub async fn get(key: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, static_method_of = KvJs, js_name = kvPut)]
    pub async fn put(key: &str, value: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, static_method_of = KvJs, js_name = kvDel)]
    pub async fn del(key: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, static_method_of = KvJs, js_name = kvKeys)]
    pub async fn keys(
        gte: Option<&str>,
        lt: Option<&str>,
        reverse: Option<bool>,
        limit: Option<u32>,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, static_method_of = KvJs, js_name = kvMap)]
    pub async fn map(
        gte: Option<&str>,
        lt: Option<&str>,
        reverse: Option<bool>,
        limit: Option<u32>,
    ) -> Result<JsValue, JsValue>;
}

pub struct Kv;

#[async_trait(?Send)]
impl KvStorage for Kv {
    async fn put<T: Serialize>(key: &str, value: &T) {
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        let value = value.serialize(&serializer).unwrap();

        KvJs::put(key, value).await.unwrap();
    }

    async fn del(key: &str) {
        KvJs::del(key).await.unwrap();
    }

    async fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
        let value = KvJs::get(key).await.ok()?;

        serde_wasm_bindgen::from_value::<T>(value).ok()
    }

    async fn map<T: DeserializeOwned>(
        gte: Option<&str>,
        lt: Option<&str>,
        reverse: Option<bool>,
        limit: Option<u32>,
    ) -> Vec<(String, T)> {
        serde_wasm_bindgen::from_value(KvJs::map(gte, lt, reverse, limit).await.unwrap()).unwrap()
    }

    async fn keys(
        gte: Option<&str>,
        lt: Option<&str>,
        reverse: Option<bool>,
        limit: Option<u32>,
    ) -> Vec<String> {
        serde_wasm_bindgen::from_value(KvJs::keys(gte, lt, reverse, limit).await.unwrap()).unwrap()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
