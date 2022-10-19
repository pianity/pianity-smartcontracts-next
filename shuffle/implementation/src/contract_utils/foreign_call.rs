use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::JsValue;

use warp_shuffle::error::{ForeignReadError, ForeignWriteError};

use crate::contract_utils::js_imports::SmartWeave;

pub async fn read_foreign_contract_state<T: DeserializeOwned>(
    contract_address: &String,
) -> Result<T, ForeignReadError> {
    let state: T = SmartWeave::read_contract_state(contract_address)
        .await
        .into_serde()
        .map_err(|_err| ForeignReadError::ParseError)?;

    Ok(state)
}

pub async fn write_foreign_contract<
    RESULT: DeserializeOwned + std::fmt::Debug,
    ERROR: Serialize + DeserializeOwned + std::fmt::Debug,
    INPUT: Serialize,
>(
    contract_address: &String,
    input: INPUT,
) -> Result<RESULT, ForeignWriteError<ERROR>> {
    let input = JsValue::from_serde(&input).unwrap();

    SmartWeave::write(contract_address, input)
        .await
        .map_err(|err| {
            let into_serde: ForeignWriteError<ERROR> = match err.into_serde::<ERROR>() {
                Ok(contract_error) => ForeignWriteError::ContractError(contract_error),
                Err(_serde_error) => ForeignWriteError::ParseError,
            };

            into_serde
        })
        .map(|result| result.into_serde().unwrap())
}
