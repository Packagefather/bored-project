use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    Version(#[from] cw2::VersionError),

    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("token_id already claimed")]
    Claimed {},

    #[error("The token_id is already staked in this collection")]
    AlreadyStakedTokenIDInCollection {},

    #[error("Staking is not allowed at the moment")]
    StakingNotAllowed {},

    #[error("Invalid number of days")]
    InvalidDaysNumber {},

    #[error("The collection is not supported")]
    InvalidCollection {},
}
