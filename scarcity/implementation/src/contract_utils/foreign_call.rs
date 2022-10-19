use crate::contract_utils::js_imports::SmartWeave;
use serde::de::DeserializeOwned;
use serde::Serialize;
use warp_scarcity::error::ForeignWriteError;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

use super::js_imports::log;

pub async fn read_foreign_contract_state<T: DeserializeOwned>(contract_address: &String) -> T {
    let state: T = SmartWeave::read_contract_state(contract_address)
        .await
        .into_serde()
        .unwrap(); // TODO: not sure if it won't case panics. Maybe it's better to return Result<T, ContractError>

    return state;
}

// #[derive(Debug)]
// pub enum ForeignWriteError<T: DeserializeOwned + std::fmt::Debug> {
//     ContractError(T),
//     ParseError(serde_json::Error),
// }

// impl<T: DeserializeOwned + std::fmt::Debug> std::fmt::Display for ForeignWriteError<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match *self {
//             Self::ContractError(_) => write!(f, "please use a vector with at least one element"),
//             // The wrapped error contains additional information and is available
//             // via the source() method.
//             Self::ParseError(_) => write!(f, "the provided string could not be parsed as int"),
//         }
//     }
// }
//
// impl<T: DeserializeOwned + std::fmt::Debug> std::error::Error for ForeignWriteError<T> {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match *self {
//             Self::ContractError(_) => None,
//             // The cause is the underlying implementation error type. Is implicitly
//             // cast to the trait object `&error::Error`. This works because the
//             // underlying type already implements the `Error` trait.
//             Self::ParseError(ref e) => Some(e),
//         }
//     }
// }
//
// impl<T: DeserializeOwned + std::fmt::Debug> From<serde_json::Error> for ForeignWriteError<T> {
//     fn from(err: serde_json::Error) -> ForeignWriteError<T> {
//         ForeignWriteError::ParseError(err)
//     }
// }

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
