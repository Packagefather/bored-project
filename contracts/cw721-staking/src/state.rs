use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, BlockInfo, CustomMsg, StdResult, Storage};

use cw721::{ContractInfoResponse, Cw721, Expiration};
use cw_storage_plus::{Item, Map};


pub const CONFIG: Item<Config> = Item::new("config");

pub const STATE: Item<State> = Item::new("state");

//pub const NFT_STAKES: Map<&Addr, StakeInfo> = Map::new("nft_stakes"); //We need to rewrite this or something else, the token_ids cannot repeat
//yet they may repeat for different collections
// Stores token_id and collection address in a turple as key and StakeInfo as value
pub const NFT_STAKES: Map<(String, &Addr), StakeInfo> = Map::new("nft_stakes");


pub const REWARD_RATE: Map<u64, u64> = Map::new("reward_rate");

pub const COLLECTIONS: Map<&Addr, bool> = Map::new("collections");

pub const USER_INFO: Map<Addr, UsesrInfo> = Map::new("user_info");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct StakeInfo {
    
    pub owner: Addr,
   
    pub total_earned_tokens: Uint128,
    
    pub token_id: String,
   
    pub stake_days: u64,

    pub reward_rate: u64,

    pub last_claim_time: u64,

    pub staked: bool,

    pub collection: Addr,


}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[derive(Default)]
pub struct State {
    pub total_stakes: Uint128,
    pub total_claims: Uint128,
    pub funded_tokens: Uint128,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub token_address: Addr,
    pub is_staking_allowed: bool,
}