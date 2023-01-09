use cosmwasm_std::{Addr, Uint128, Uint64, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    WithdrawFee {
        // release some coins - if quantity is None, release all coins in balance
        to: Addr,
        amount: Uint128,
    },
    SetAdmin {
        new_admin: Addr,
    },
    SetBotRole {
        new_bot: Addr,
        enabled: bool
    },
    BuyToken { 
        juno_amount: Uint128
        , token_amount_per_native: Uint128
        , slippage_bips: Uint128
        , recipient: Addr
        , pool_address: Addr
        , platform_fee_bips: Uint128
        , gas_estimate: Uint128
        , deadline: Uint64
    },
    SwapAtomToJuno {
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns a human-readable representation of the arbiter.
    GetInfos {
        token: String,
    },    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminResponse {
    pub admin: Addr,
    pub pending_platform_fee: Uint128,
    pub blocktime: u64,
    pub token_balance: Uint128,
    pub token_balances: Vec<Coin>,
    pub contract_address: Addr,
    //pub all_tokens: Vec<Coin>
}

pub struct BotsResponse {
    pub admin: String,
}


