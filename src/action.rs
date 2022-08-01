use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::actions::transfer::Transfer;
use crate::contract_utils::handler_result::HandlerResult as HandlerResultGeneric;
use crate::error::ContractError;
use crate::state::{Balance, BalancePrecision, State};

// function safeTransferFrom(address _from, address _to, uint256 _id, uint256 _value, bytes calldata _data) external;
//
// function safeBatchTransferFrom(address _from, address _to, uint256[] calldata _ids, uint256[] calldata _values, bytes calldata _data) external;
//
// function balanceOf(address _owner, uint256 _id) external view returns (uint256);
//
// function balanceOfBatch(address[] calldata _owners, uint256[] calldata _ids) external view returns (uint256[] memory);
//
// function setApprovalForAll(address _operator, bool _approved) external;
//
// function isApprovedForAll(address _owner, address _operator) external view returns (bool);

pub trait Actionable {
    fn action(self, caller: String, state: State) -> ActionResult;
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConfigureArgs {
    pub super_owner: Option<String>,
    pub owners: Option<Vec<String>>,
    pub authorized_addresses: Option<Vec<String>>,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BatchArgs {
    pub actions: Vec<Action>,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MintArgs {
    pub ticker: Option<String>,
    pub prefix: Option<String>,
    pub qty: Balance,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase", tag = "function")]
#[ts(export)]
pub enum Action {
    #[serde(rename_all = "camelCase")]
    Transfer(Transfer),

    #[serde(rename_all = "camelCase")]
    BalanceOf {
        token_id: String,
        target: String,
    },

    Configure(ConfigureArgs),

    SetApprovalForAll {
        operator: String,
        approved: bool,
    },

    IsApprovedForAll {
        owner: String,
        operator: String,
    },

    Evolve {
        value: String,
    },

    Mint(MintArgs),

    Batch(BatchArgs),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ReadResponse {
    Balance {
        balance: BalancePrecision,
        target: String,
    },

    ApprovedForAll {
        approved: bool,
        owner: String,
        operator: String,
    },

    Batch(Vec<ReadResponse>),
}

pub type HandlerResult = HandlerResultGeneric<State, ReadResponse>;
pub type ActionResult = Result<HandlerResult, ContractError>;
