use crate::contract_utils::js_imports::SmartWeave;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub async fn read_foreign_contract_state<T: DeserializeOwned>(contract_address: &str) -> T {
    serde_wasm_bindgen::from_value(SmartWeave::read_contract_state(contract_address).await).unwrap()
}

pub async fn write_foreign_contract<T: DeserializeOwned, I: Serialize>(
    contract_address: &str,
    input: I,
) -> T {
    serde_wasm_bindgen::from_value(
        SmartWeave::write(
            contract_address,
            serde_wasm_bindgen::to_value(&input).unwrap(),
        )
        .await,
    )
    .unwrap()
}
