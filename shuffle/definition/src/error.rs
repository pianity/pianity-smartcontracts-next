use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "data")]
pub enum ForeignReadError {
    // ReadError,
    ParseError,
}

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "data")]
pub enum ForeignWriteError<T: Serialize + DeserializeOwned + std::fmt::Debug> {
    #[serde(deserialize_with = "T::deserialize")]
    ContractError(T),
    ParseError,
}

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "data")]
pub enum ContractError {
    RuntimeError(String),
    TransferAmountMustBeHigherThanZero,
    TransferFromAndToCannotBeEqual,
    TokenNotFound(String),
    IDontLikeThisContract,
    CallerBalanceNotEnough(u64),
    OnlyOwnerCanEvolve,
    EvolveNotAllowed,

    ForbiddenNestedBatch,
    CannotMixeReadAndWrite,
    EmptyBatch,

    UnauthorizedConfiguration,
    UnauthorizedAddress(String),
    UnauthorizedTransfer(String),

    InvalidFee,
    InvalidRate,

    TokenOwnerNotFound,

    TokenAlreadyExists(String),
    TokenDoesNotExist(String),
    TokenIsNotAnNFT(String),

    TransferResult(String),

    Erc1155Error(ForeignWriteError<warp_erc1155::error::ContractError>),
    Erc1155ReadFailed,

    ShuffleNotFound(String),
    /// (shuffle_id, nft_id)
    NftAlreadyInAShuffle(String, String),
    /// (shuffle_id)
    NoNftAvailable(String),

    BoostCapExceeded,
}
