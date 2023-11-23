use cosmwasm_std::{Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VestingDetails {
    // start_time: after this timestamp vesting will start
    pub start_time: u64,
    // List of intervals and amount, after each interval certain amount will be released
    pub schedules: Vec<ReleaseInterval>,
    // token receiver, can claim tokens
    pub receiver: String,
    // total amount of tokens,
    pub token: Coin,
    // total claimed
    pub amount_claimed: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ReleaseInterval {
    pub interval: u64,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    // admin
    pub admin: String,
    // allowed addresses which can enable vesting for receiver
    pub allowed_addresses: Vec<String>,
}

// Map from (User, OrderId) -> VestingDetails
pub const VESTED_TOKENS_ALL: Map<(String, String), VestingDetails> = Map::new("vested_tokens_all");

pub const CONFIG: Item<Config> = Item::new("config");
