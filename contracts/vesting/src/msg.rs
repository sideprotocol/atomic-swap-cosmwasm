use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::VestingDetails;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub allowed_addresses: Vec<String>,
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
        order_id: String,
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
    QueryVestingDetails { address: String, order_id: String },
    /// Returns config
    QueryConfig {},
}
