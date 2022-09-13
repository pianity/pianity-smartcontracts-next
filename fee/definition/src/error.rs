use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ForeignWriteError<T: Serialize + DeserializeOwned + std::fmt::Debug> {
    #[serde(deserialize_with = "T::deserialize")]
    ContractError(T),
    ParseError,
}

#[derive(Serialize, Deserialize, Debug)]
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
}
