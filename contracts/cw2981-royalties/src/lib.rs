pub mod error;
pub mod msg;
pub mod query;

pub use query::{check_royalties, query_royalties_info};

use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Empty, Timestamp};
use tokio::time::{sleep, Duration};

use cw721_base::Cw721Contract;
pub use cw721_base::{InstantiateMsg, MinterResponse};
pub use cw721::{Expiration, UserOfResponse};
//pub use cw721::Expiration::AtTime;
use crate::error::ContractError;
use crate::msg::Cw2981QueryMsg;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw2981-royalties";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[cw_serde]
#[derive(Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    /// This is how much the minter takes as a cut when sold
    /// royalties are owed on this token if it is Some
    pub royalty_percentage: Option<u64>,
    /// The payment address, may be different to or the same
    /// as the minter addr
    /// question: how do we validate this?
    pub royalty_payment_address: Option<String>,
}

pub type Extension = Option<Metadata>;

pub type MintExtension = Option<Extension>;

pub type Cw2981Contract<'a> = Cw721Contract<'a, Extension, Empty, Empty, Cw2981QueryMsg>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;
pub type QueryMsg = cw721_base::QueryMsg<Cw2981QueryMsg>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        Ok(Cw2981Contract::default().instantiate(deps.branch(), env, info, msg)?)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        if let ExecuteMsg::Mint {
            extension:
                Some(Metadata {
                    royalty_percentage: Some(royalty_percentage),
                    ..
                }),
            ..
        } = &msg
        {
            // validate royalty_percentage to be between 0 and 100
            // no need to check < 0 because royalty_percentage is u64
            if *royalty_percentage > 100 {
                return Err(ContractError::InvalidRoyaltyPercentage);
            }
        }

        Cw2981Contract::default()
            .execute(deps, env, info, msg)
            .map_err(Into::into)
    }

    #[entry_point] 
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Extension { msg } => match msg {
                Cw2981QueryMsg::RoyaltyInfo {
                    token_id,
                    sale_price,
                } => to_json_binary(&query_royalties_info(deps, token_id, sale_price)?),
                Cw2981QueryMsg::CheckRoyalties {} => to_json_binary(&check_royalties(deps)?),
            },
            _ => Cw2981Contract::default().query(deps, env, msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse};

    use cosmwasm_std::{from_json, Uint128};

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721::Cw721Query;

    const CREATOR: &str = "creator";
    const BADACTOR: &str = "badactor";
    const JOHN: &str = "john";

    #[test]
    fn use_metadata_extension() {
        let mut deps = mock_dependencies();
        let contract = Cw2981Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: None,
            withdraw_address: None,
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let token_uri = Some("https://fuchsia-vague-hookworm-215.mypinata.cloud/ipfs/Qmatfw5QbRWNyUyWEtn6y3QizS8sQYogo7RPd341tv6Mdp".into());
        let extension = Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        });
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: token_uri.clone(),
            extension: extension.clone(),
        };
        entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        assert_eq!(res.token_uri, token_uri);
        assert_eq!(res.extension, extension);
    }

    #[test]
    fn validate_royalty_information() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: None,
            withdraw_address: None,
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                royalty_percentage: Some(101),
                ..Metadata::default()
            }),
        };
        // mint will return StdError
        let err = entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidRoyaltyPercentage);
    }

    #[test]
    fn check_royalties_response() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981Contract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: None,
            withdraw_address: None,
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();

        let expected = CheckRoyaltiesResponse {
            royalty_payments: true,
        };
        let res = check_royalties(deps.as_ref()).unwrap();
        assert_eq!(res, expected);

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981QueryMsg::CheckRoyalties {},
        };
        let query_res: CheckRoyaltiesResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);
    }

    #[test]
    fn check_token_royalties() {
        let mut deps = mock_dependencies();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: None,
            withdraw_address: None,
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let owner = "jeanluc";
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: owner.into(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                royalty_payment_address: Some("jeanluc".to_string()),
                royalty_percentage: Some(10),
                ..Metadata::default()
            }),
        };
        entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

        let expected = RoyaltiesInfoResponse {
            address: owner.into(),
            royalty_amount: Uint128::new(10),
        };
        let res =
            query_royalties_info(deps.as_ref(), token_id.to_string(), Uint128::new(100)).unwrap();
        assert_eq!(res, expected);

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981QueryMsg::RoyaltyInfo {
                token_id: token_id.to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);

        // check for rounding down
        // which is the default behaviour
        let voyager_token_id = "Voyager";
        let owner = "janeway";
        let voyager_exec_msg = ExecuteMsg::Mint {
            token_id: voyager_token_id.to_string(),
            owner: owner.into(),
            token_uri: Some("https://starships.example.com/Starship/Voyager.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Voyager".to_string()),
                royalty_payment_address: Some("janeway".to_string()),
                royalty_percentage: Some(4),
                ..Metadata::default()
            }),
        };
        entry::execute(deps.as_mut(), mock_env(), info, voyager_exec_msg).unwrap();

        // 43 x 0.04 (i.e., 4%) should be 1.72
        // we expect this to be rounded down to 1
        let voyager_expected = RoyaltiesInfoResponse {
            address: owner.into(),
            royalty_amount: Uint128::new(1),
        };

        let res = query_royalties_info(
            deps.as_ref(),
            voyager_token_id.to_string(),
            Uint128::new(43),
        )
        .unwrap();
        assert_eq!(res, voyager_expected);
    }

    #[test]
    fn test_setting_tokenid_user() {

        let mut deps = mock_dependencies();
        let contract = Cw2981Contract::default();
        let mut env = mock_env();

        let info = mock_info(CREATOR, &[]);
        let badactor = mock_info(BADACTOR, &[]);
        let john = mock_info(JOHN, &[]);


        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: None,
            withdraw_address: None,
        };
        entry::instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let token_id = "1";
        let token_uri = Some("https://fuchsia-vague-hookworm-215.mypinata.cloud/ipfs/Qmatfw5QbRWNyUyWEtn6y3QizS8sQYogo7RPd341tv6Mdp".into());
        let extension = Some(Metadata {
            description: Some("Spaceship with Warp Drive".into()),
            name: Some("Starship USS Enterprise".to_string()),
            ..Metadata::default()
        });
        let exec_msg = ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: JOHN.clone().to_string(),
            token_uri: token_uri.clone(),
            extension: extension.clone(),
        };
        entry::execute(deps.as_mut(), env.clone(), info.clone(), exec_msg).unwrap();

        let res = contract.nft_info(deps.as_ref(), token_id.into()).unwrap();
        println!("The nft info: {:?}", res);
        assert_eq!(res.token_uri, token_uri);
        assert_eq!(res.extension, extension);

        //Time is read here in nanoseconds not seconds
        //Timestamp: A point in time in nanosecond precision.
        //More info: https://docs.rs/cosmwasm-std/1.5.3/src/cosmwasm_std/timestamp.rs.html#37-39
        
        let expiresTime = env.block.time.plus_seconds(60); //1571797419 
        println!("Current Block time one: {:?}", env.block.time); 
        println!("Set expiry time: {:?}", expiresTime); 

        let start = tokio::time::Instant::now();
        println!("Current Block time Tokio: {:?}", start); 

        {/*
            //This is definition of plus_seconds
            pub const fn plus_seconds(&self, addition: u64) -> Timestamp {
            self.plus_nanos(addition * 1_000_000_000)
        */}

        //This creates a timestamp from nanosecond value given, Result => Timestamp(Uint64(15717976619))
        //let ts = Timestamp::from_nanos(15717976619); 
        //println!("Convert from nanos: {:?}", ts);

        let exec_msg = ExecuteMsg::SetUser{
            token_id: token_id.clone().to_string(),
            user: "testuser".to_string(), 
            expires: Some(Expiration::AtTime(expiresTime))
        }; 
    
        let res = entry::execute(deps.as_mut(), env.clone(), john.clone(), exec_msg).unwrap();
        println!("set user function: {:?}", res); 

       

        let res = contract.user_of(deps.as_ref(), env.clone(), token_id.to_string()).unwrap();
        //To convert the timestamp from nanoseconds to seconds, you would divide by 1,000,000,000
        println!("User of an NFT: {:?}", res.expires.to_string()); //User of an NFT: "expiration time: 1571797479.879305533"
        assert_eq!(res.expires.is_expired(&env.block), false);

        // let nanoseconds = match &res.expires {
        //     Expiration::AtTime(timestamp) => timestamp.seconds(),
        //     _ => 0,  // Handle other cases if needed
        // };
        // println!("Nanoseconds: {}", nanoseconds);

        /*
        let mut router: App = App::new(|_, _, _| {});
            router.update_block(|current_blockinfo| {
            current_blockinfo.height += 1;
            current_blockinfo.time = current_blockinfo.time.plus_seconds(2592000); //30 days
        }); */

        //Fast forward by 30days
        //env.block.time += Timestamp::plus_seconds(2592000);
        env.block.time = env.block.time.plus_seconds(2592000);

        println!("Current Block time two: {:?}", env.block.time); 
         // also check the longhand way
        let query_msg = QueryMsg::UserOf{
            token_id: token_id.clone().to_string()
        };
        let query_res: UserOfResponse =
            from_json(entry::query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        //assert_eq!(query_res, expected);
        println!("Result of last query: {:?}", query_res);
        //println!("Result of last query: {:?}", query_res.expires.is_expired(&env.block));
        assert_eq!(query_res.expires.is_expired(&env.block), true)

        //1571797419.879305533
        //1574389419.879305533

       
    }
}
