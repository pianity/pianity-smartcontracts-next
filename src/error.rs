use serde::Serialize;

use crate::state::BalancePrecision;

#[derive(Serialize, Debug)]
pub enum ContractError {
    RuntimeError(String),
    TransferAmountMustBeHigherThanZero,
    TransferFromAndToCannotBeEqual,
    TokenNotFound(String),
    IDontLikeThisContract,
    CallerBalanceNotEnough(BalancePrecision),
    OnlyOwnerCanEvolve,
    EvolveNotAllowed,

    ForbiddenNestedBatch,
    CannotMixeReadAndWrite,
    EmptyBatch,

    UnauthorizedConfiguration,
    UnauthorizedAddress(String),
    UnauthorizedTransfer(String),
    TokenAlreadyExists,
}
