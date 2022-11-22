use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "data")]
pub enum ForeignReadError {
    // ReadError,
    ParseError,
}

#[derive(JsonSchema, Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "data")]
pub enum ForeignWriteError<T> {
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

    InvalidRoyalties,
    InvalidRate,

    TokenOwnerNotFound,

    TokenAlreadyExists(String),
    TokenDoesNotExist(String),
    TokenIsNotAnNFT(String),
    InvalidTokenId,

    TransferResult(String),

    Erc1155ReadFailed,
    Erc1155Error(ForeignWriteError<warp_erc1155::error::ContractError>),
    InvalidNftId(String),

    QtyMustBeOneForNftTransfers,

    ContractIsPaused,
}
