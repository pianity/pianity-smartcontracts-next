use std::collections::HashMap;

use serde::Serialize;
use serde::{de::DeserializeOwned, Deserialize};
use wasm_bindgen::JsValue;

use warp_lock::error::{ForeignReadError, ForeignWriteError};

use crate::contract_utils::js_imports::SmartWeave;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ForeignContractState {
    Erc1155(warp_erc1155::state::State),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultOk {
    state: ForeignContractState,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultError<ERROR> {
    error: ERROR,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ForeignCallResult<ERROR> {
    Ok(ResultOk),
    Error(ResultError<ERROR>),
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalWriteResult {
    #[serde(rename = "type")]
    result_type: String,
    state: ForeignContractState,
}

pub struct ForeignContractCaller {
    states: HashMap<String, ForeignContractState>,
}

impl ForeignContractCaller {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub async fn read(
        &mut self,
        contract_address: &String,
    ) -> Result<&ForeignContractState, ForeignReadError> {
        if !self.states.contains_key(contract_address) {
            let state = SmartWeave::read_contract_state(contract_address)
                .await
                .into_serde()
                .map_err(|_err| ForeignReadError::ParseError)?;

            self.states.insert(contract_address.to_string(), state);
        }

        Ok(self.states.get(contract_address).unwrap())
    }

    pub async fn write<ERROR: Serialize + DeserializeOwned + std::fmt::Debug, INPUT: Serialize>(
        &mut self,
        contract_address: &String,
        input: INPUT,
    ) -> Result<&ForeignContractState, ForeignWriteError<ERROR>> {
        let input = JsValue::from_serde(&input).unwrap();

        let result = SmartWeave::write(contract_address, input)
            .await
            .into_serde::<ForeignCallResult<ERROR>>()
            .map_err(|_err| ForeignWriteError::ParseError)?;

        match result {
            ForeignCallResult::Ok(state) => {
                self.states
                    .insert(contract_address.to_string(), state.state);
                Ok(self.states.get(contract_address).unwrap())
            }
            ForeignCallResult::Error(error) => Err(ForeignWriteError::ContractError(error.error)),
        }
    }
}
