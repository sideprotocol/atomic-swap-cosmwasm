use cw721_base::state::Approval;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::VestingDetails;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub allowed_addresses: Vec<String>,
    pub token_code_id: u64,
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ExecuteMsg {
    StartVesting {
        vesting: VestingDetails,
        order_id: String,
    },
    SetAllowed {
        addresses: Vec<String>,
    },
    Claim {
        nft_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns all claims details
    // QueryClaims { address: String },
    /// Returns all vesting details
    QueryVestingDetails { nft_id: String },
    /// Returns config
    QueryConfig {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerOfResponse {
    /// Owner of the token
    pub owner: String,
    /// If set this address is approved to transfer/send the token as well
    pub approvals: Vec<Approval>,
}
