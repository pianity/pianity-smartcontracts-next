use serde::Serialize;

#[derive(Serialize, Debug)]
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
}
