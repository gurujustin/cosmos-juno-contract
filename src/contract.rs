use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, QuerierWrapper,
    Uint128, Uint64, CosmosMsg,
    StdResult,
};

use cw20::Denom;

use crate::error::ContractError;
use crate::msg::{AdminResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config, config_read, State, BOT_ROLES};
use crate::util;

//const GAS_MAX: u128 = 2000u128;
const ATOM_DENOM: &str = "ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9"; //ibc atom token
const ATOM_JUNO_POOL_ADDR: &str = "juno1sg6chmktuhyj4lsrxrrdflem7gsnk4ejv6zkcc4d3vcqulzp55wsf4l4gl";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        pending_platform_fee: Uint128::zero(),
    };

    config(deps.storage).save(&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut state = config_read(deps.storage).load()?;
    match msg {
        ExecuteMsg::SetAdmin { new_admin } => try_set_admin(deps, &mut state, info, new_admin),
        ExecuteMsg::SetBotRole { new_bot, enabled } => try_set_bot_role(deps, state, info, new_bot, enabled),
        ExecuteMsg::BuyToken {juno_amount, token_amount_per_native, slippage_bips, recipient, pool_address, platform_fee_bips, gas_estimate, deadline} => 
                buy_token(deps, &mut state, info, env, juno_amount, token_amount_per_native, slippage_bips, recipient, pool_address, platform_fee_bips, gas_estimate, deadline),      
        ExecuteMsg::WithdrawFee { to, amount } => try_withdraw_fee(deps, &mut state, info, to, amount),
        ExecuteMsg::SwapAtomToJuno {} => try_swap_atom(deps, &mut state, env, info),
    }
}

fn try_swap_atom(    
    deps: DepsMut,
    _state: &mut State,
    env: Env,
    _info: MessageInfo,
)-> Result<Response, ContractError> {
    let messags = get_message_swap_atom(deps.querier, env,  String::from(ATOM_DENOM), Addr::unchecked(ATOM_JUNO_POOL_ADDR))?;

    Ok(Response::new()
        .add_messages(messags))
}

fn get_message_swap_atom(    
    querier: QuerierWrapper,
    env: Env,
    token: String,
    pool_address: Addr
)-> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];
    
    let token_balance = util::get_token_amount(querier, Denom::Native(token.clone()), env.contract.address)?;

    if token_balance == Uint128::zero() {
        return Ok(messages);
    }

    let (_token2_amount, _token2_denom, mut messages_swap) = 
        util::get_swap_amount_and_denom_and_message(querier
            , pool_address
            , Denom::Native(token)
            , token_balance
            , Uint128::zero()
            , None)?;
    messages.append(&mut messages_swap);    

    Ok(messages)
}

fn try_set_admin(
    deps: DepsMut,
    state: &mut State,
    info: MessageInfo,
    new_admin: Addr
) -> Result<Response, ContractError> {
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized { });
    }

    state.owner = new_admin.clone();
    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("new_admin", new_admin)
    )
}

fn try_set_bot_role(
    deps: DepsMut,
    state: State,
    info: MessageInfo,
    new_bot: Addr,
    role: bool
) -> Result<Response, ContractError> {
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized { });
    }

    BOT_ROLES.save(deps.storage, new_bot.clone(), &role)?;
    
    Ok(Response::new()
        .add_attribute("bot added", "Yes")
    )
}

fn try_withdraw_fee(
    deps: DepsMut,
    state: &mut State,
    info: MessageInfo,
    to: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {
    if state.owner != info.sender {
        return Err(ContractError::Unauthorized { });
    }

    state.pending_platform_fee -= amount;

    config(deps.storage).save(&state)?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    msgs.push(util::transfer_token_message(Denom::Native(String::from("ujuno")), amount, to)?);

    Ok(Response::new()
        .add_messages(msgs)
    )
}

fn buy_token(
    deps: DepsMut,
    state: &mut State,
    info: MessageInfo,
    env: Env,
    juno_amount: Uint128,
    token_amount_per_native: Uint128,
    slippage_bips: Uint128,
    recipient: Addr,
    pool: Addr,
    platform_fee_bips: Uint128,
    gas_estimate: Uint128,
    deadline: Uint64,
) -> Result<Response, ContractError> {
    
    if !BOT_ROLES.has(deps.storage, info.sender.clone()) {
        return Err(ContractError::Unauthorized {});    
    }
    let enabled = BOT_ROLES.load(deps.storage, info.sender)?;
    if !enabled {
        return Err(ContractError::UnauthorizedRole {});    
    }

    if env.block.time.seconds() > deadline.u64() {
        return Err(ContractError::Expired { });
    }

    if slippage_bips > Uint128::from(10000u128) {
        return Err(ContractError::BuyingUtilityOverSlippages { });
    }

    if gas_estimate > juno_amount {
        return Err(ContractError::InsufficientToken{});
    }

    let mut messages = get_message_swap_atom(deps.querier
                         , env,  String::from(ATOM_DENOM), Addr::unchecked(ATOM_JUNO_POOL_ADDR))?;

    let mut _juno_amount = juno_amount - gas_estimate;

    let platform_fee = platform_fee_bips * juno_amount / Uint128::from(10000u128);
    state.pending_platform_fee += platform_fee;
    //let approxTxFee = gas_estimate * tx.gasprice;
    let amount_out_min = _juno_amount * token_amount_per_native * (Uint128::from(10000u128) - slippage_bips) / Uint128::from(10000000000u128);
    _juno_amount -= platform_fee;

    if _juno_amount <= Uint128::zero() {
        return Err(ContractError::InsufficientEthToSwap{});
    }

    let (_token2_amount, _token2_denom, mut messages_swap) = 
        util::get_swap_amount_and_denom_and_message(deps.querier
            , pool
            , Denom::Native(String::from("ujuno"))
            , juno_amount
            , amount_out_min
            , Some(recipient))?;
    messages.append(&mut messages_swap);    

    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_messages(messages))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInfos {token} => to_binary(&query_infos(deps, env, token)?),
    }
}

fn query_infos(deps: Deps, env: Env, token: String) -> StdResult<AdminResponse> {
    let state = config_read(deps.storage).load()?;
    let admin = state.owner;
    let pending_platform_fee = state.pending_platform_fee;
    let blocktime = env.block.time.seconds();
    let contract_address = env.contract.address.clone();
    let token_balance = util::get_token_amount(deps.querier, Denom::Native(token), env.contract.address.clone())?;
    let token_balances = util::get_tokens_amounts(deps.querier, env.contract.address)?;
    
    Ok(AdminResponse { admin, pending_platform_fee, blocktime, token_balance, token_balances, contract_address })
}

