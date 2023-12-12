#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo,
    Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::InstantiateMsg as Cw721InstantiateMsg,
    msg::QueryMsg as Cw721QueryMsg,
};

use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, VestingDetails, CONFIG, VESTED_TOKENS_ALL};

// Version info, for migration info
const CONTRACT_NAME: &str = "vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let config = Config {
        admin: info.sender.into_string(),
        allowed_addresses: msg.allowed_addresses,
        cw721_address: None,
        extension: msg.extension,
    };
    CONFIG.save(deps.storage, &config)?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_binary(&Cw721InstantiateMsg {
                name: msg.name.clone(),
                symbol: msg.symbol,
                minter: env.contract.address.to_string(),
            })?,
            funds: vec![],
            admin: None,
            label: String::from("Instantiate NFT contract"),
        }
        .into(),
        id: INSTANTIATE_TOKEN_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new().add_submessages(sub_msg))
}

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.cw721_address.is_some() {
        return Err(ContractError::Cw721AlreadyLinked {});
    }

    if msg.id != INSTANTIATE_TOKEN_REPLY_ID {
        return Err(ContractError::InvalidTokenReplyId {});
    }

    let reply = parse_reply_instantiate_data(msg).unwrap();
    config.cw721_address = Addr::unchecked(reply.contract_address).into();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StartVesting { vesting, order_id } => {
            execute_start_vesting(deps, env, info, vesting, order_id)
        }
        ExecuteMsg::SetAllowed { addresses } => execute_set_contract(deps, env, info, addresses),
        ExecuteMsg::Claim { nft_id } => execute_claim(deps, env, info, nft_id),
    }
}

pub fn execute_start_vesting(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    vesting: VestingDetails,
    order_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut ok = false;

    if config.cw721_address.is_none() {
        return Err(ContractError::Cw721NotLinked {});
    }

    for address in config.allowed_addresses {
        if address == info.sender {
            ok = true;
        }
    }
    if !ok {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Must be called by allowed address"
        ))));
    }

    let msg = Cw721ExecuteMsg::<_, Empty>::Mint {
        token_id: order_id.clone(),
        owner: vesting.receiver.clone(),
        token_uri: None,
        extension: config.extension,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.cw721_address.unwrap().into_string(),
        msg: to_binary(&msg)?,
        funds: vec![],
    };

    let mut total_amount = Uint128::from(0u64);
    for schedule in vesting.schedules.clone() {
        total_amount += schedule.amount;
    }
    if total_amount != vesting.token.amount.clone() {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Total amount of tokens is not equal to total vesting amount"
        ))));
    }

    // check if given tokens are received here
    let mut ok = false;
    // First token in this chain only first token needs to be verified
    for asset in info.funds {
        if asset == vesting.token.clone() {
            ok = true;
        }
    }
    if !ok {
        return Err(ContractError::Std(StdError::generic_err(
            "Funds mismatch: Funds mismatched to with message and sent values: Start vesting"
                .to_string(),
        )));
    }

    VESTED_TOKENS_ALL.save(deps.storage, order_id, &vesting)?;

    let res = Response::new()
        .add_message(exec)
        .add_attribute("action", "start_vesting");
    Ok(res)
}

pub fn execute_set_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addresses: Vec<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Must be called by admin"
        ))));
    }

    config.allowed_addresses = addresses;
    CONFIG.save(deps.storage, &config)?;

    let res = Response::new().add_attribute("action", "set_allowed_addresses");
    Ok(res)
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nft_id: String,
) -> Result<Response, ContractError> {
    let mut vesting = VESTED_TOKENS_ALL.load(deps.storage, nft_id.clone())?;

    // TODO: Add check for owner of nft_id, nft_id owner can claim
    let mut send_msg = vec![];

    let now = env.block.time.seconds();
    if vesting.start_time <= now {
        let mut now_with_cliff = vesting.start_time;
        let mut release_amount = Uint128::from(0u64);
        for schedule in vesting.schedules.clone() {
            now_with_cliff += schedule.interval;
            if now_with_cliff <= now {
                release_amount += schedule.amount;
            } else {
                break;
            }
        }
        let final_amount = release_amount.u128() - vesting.amount_claimed.u128();

        send_msg.push(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: vesting.token.denom.clone(),
                amount: Uint128::from(final_amount),
            }],
        });

        vesting.amount_claimed += Uint128::from(final_amount);
    }

    VESTED_TOKENS_ALL.save(deps.storage, nft_id, &vesting)?;

    let res = Response::new()
        .add_attribute("action", "claim_tokens")
        .add_messages(send_msg);
    Ok(res)
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ver = cw2::get_contract_version(deps.storage)?;
    // ensure we are migrating from an allowed contract
    if ver.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Can only upgrade from same type").into());
    }
    // note: better to do proper semver compare, but string compare *usually* works
    if ver.version >= CONTRACT_VERSION.to_string() {
        return Err(StdError::generic_err("Cannot upgrade from a newer version").into());
    }

    // set the new version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        //QueryMsg::QueryClaims { address } => to_json_binary(&query_claims(deps, address)?),
        QueryMsg::QueryConfig {} => to_json_binary(&query_config(deps)?),
        QueryMsg::QueryVestingDetails { nft_id } => {
            to_json_binary(&query_vesting_details(deps, nft_id)?)
        }
    }
}

// fn query_claims(deps: Deps, address: String) -> StdResult<String> {
//     let config = CONFIG.load(deps.storage)?;

//     Ok(config.contract_address)
// }

fn query_vesting_details(deps: Deps, nft_id: String) -> StdResult<VestingDetails> {
    let vesting_details = VESTED_TOKENS_ALL.load(deps.storage, nft_id)?;
    Ok(vesting_details)
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}
