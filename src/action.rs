use serde::{Deserialize, Serialize};

use crate::contract_utils::handler_result::HandlerResult;
use crate::error::ContractError;
use crate::state::State;

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigureArgs {
    pub super_owner: Option<String>,
    pub owners: Option<Vec<String>>,
    pub authorized_addresses: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchArgs {
    pub actions: Vec<Action>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "function")]
pub enum Action {
    #[serde(rename_all = "camelCase")]
    Transfer {
        from: Option<String>,
        to: String,
        token_id: String,
        qty: u64,
    },

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

    Batch(BatchArgs),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum QueryResponseMsg {
    #[serde(rename_all = "camelCase")]
    Balance {
        balance: u64,
        ticker: String,
        target: String,
    },

    #[serde(rename_all = "camelCase")]
    ApprovedForAll {
        approved: bool,
        owner: String,
        operator: String,
    },
}

pub type ActionResult = Result<HandlerResult<QueryResponseMsg>, ContractError>;
