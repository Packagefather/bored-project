use cw_ownable::OwnershipError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use cosmwasm_std::{
    Addr, BankMsg, Binary, Coin, CustomMsg, Deps, DepsMut, Env, MessageInfo, Response, Storage,
};

use cw721::{ContractInfoResponse, Cw721Execute, Cw721ReceiveMsg, Expiration};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Approval, Cw721Contract, TokenInfo, User};

const CONTRACT_NAME: &str = "crates.io:staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    let owner_addr = deps.api.addr_validate(&msg.owner)?;
    let token_address = deps.api.addr_validate(&msg.token_address)?;

    let config = Config {
        owner: owner_addr,
        token_address: token_address,
        is_staking_allowed: msg.boolean,
    
};
    let state = State {
        total_sales: Uint128::new(0),
        total_stakes: Uint128::new(0),
        total_claims: Uint128::new(0),
        funded_tokens: Uint128::new(0),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::WhitelistCollection {collection_addr, boolean} => try_whitelisting_collection(deps, env, info, collection_addr, boolean),
        ExecuteMsg::AddRwardRate{days_number, rate} => try_adding_reward_rate(deps, env, info, days_number, rate),
        ExecuteMsg::Stake {token_id, number_days} => try_staking(deps, env, info, token_id, number_days),
        ExecuteMsg::Claim {token_id} => try_claim(deps, env, info, token_id),
        ExecuteMsg::Unstake {token_id} => try_unstake(deps, env, info, token_id),
        //ExecuteMsg::Withdraw {token_id} => try_withdraw(deps, env, info, token_id),
        ExecuteMsg::UpdateConfig { 
            owner,
            token_address,
            sales_wallet,
            is_staking_allowed,  

        } => try_update_config(deps, env, info,
            owner,
            token_address,
            is_staking_allowed,
        ),
    }
}

pub fn try_update_config(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    owner: String,
    token_address: String,
    is_staking_allowed: bool

    ) -> Result<Response, ContractError> {

        let mut config = CONFIG.load(deps.storage)?;

        if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
        }
        // no comment yet
        let owner_addr = deps.api.addr_validate(&owner)?;
        let token_address = deps.api.addr_validate(&token_address)?;

        config.owner = owner_addr;
        config.token_address = token_address;
        config.is_staking_allowed = is_staking_allowed;

        CONFIG.save(deps.storage, &config)?;

        Ok(Response::new()
        .add_attribute("method", "updateconfig"))
    }



pub fn try_whitelisting_collection(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    collection_addr: String,
    boolean: bool

    ) -> Result<Response, ContractError> { 

        let config = CONFIG.load(deps.storage)?;

        if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
        }

        let collection_addr = deps.api.addr_validate(&collection_addr)?;
        let  _collection = COLLECTIONS.save(deps.storage, &collection_addr.clone(), boolean.clone())?;

        Ok(Response::new()
            .add_attribute("method", "whitelist collection")
            .add_attribute("whitelist", &boolean.to_string()) 
        )
 
    } 


pub fn try_adding_reward_rate(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    number_days: u64,
    rate: u64,

    ) -> Result<Response, ContractError> { 

        let config = CONFIG.load(deps.storage)?;

        if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
        }

        let collection_addr = deps.api.addr_validate(&collection_addr)?;
        let  _collection = REWARD_RATE.save(deps.storage, number_days.clone(), rate.clone())?;

        Ok(Response::new()
            .add_attribute("method", "adding reward rate")
            .add_attribute("number_days", number_days.to_string()) 
            .add_attribute("rate", rate.to_string()) 
        )
 
    } 


//try_staking(deps, env, info, token_id, number_days),
pub fn try_staking(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo,
    token_id: String,
    number_days: u64,

    ) -> Result<Response, ContractError> { 

        let config = CONFIG.load(deps.storage)?;

        if !config.is_staking_allowed {
            return Err(ContractError::StakingNotAllowed {});
        }

        let collection_addr = deps.api.addr_validate(&collection_addr)?;
        //check that collection is whitelisted
        let  collection = COLLECTIONS.may_load(deps.storage, &collection_addr.clone())?.unwrap_or_default();
        if !collection {
            return Err(ContractError::InvalidCollection {});
        }

        let  nftstake = NFT_STAKES.may_load(deps.storage, (token_id.clone(), collection_addr))?.unwrap_or_default();

        //if exist here, error, already staked
        if nftstake.staked {
            return Err(ContractError::AlreadyStakedTokenIDInCollection {});
        }

        //check if number days is valid
        let  rewardrate = REWARD_RATE.may_load(deps.storage, number_days.clone())?.unwrap_or_default();

        if rewardrate == 0 {
            return Err(ContractError::InvalidDaysNumber {});
        }

        nftstake.owner = info.sender;
        nftstake.stake_days = number_days;
        nftstake.token_id = token_id;
        nftstake.staked = true;
        nftstake.collection = collection_addr.clone();

        NFT_STAKES.save(deps.storage, (token_id, &collection_addr), nftstake)?;
        
        Ok(Response::new()
            .add_attribute("method", "adding reward rate")
            .add_attribute("number_days", number_days.to_string()) 
            .add_attribute("rate", rate.to_string()) 
            .add_attribute("collection", collection_addr.to_string()) 
        )
 
    } 