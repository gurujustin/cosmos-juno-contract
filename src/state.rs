use cosmwasm_std::{Addr, Storage, Uint128, };
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Map};

static CONFIG_KEY: &[u8] = b"config";

pub const BOT_KEY: &str = "bot_role";
pub const BOT_ROLES: Map<Addr, bool> = Map::new(BOT_KEY);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub pending_platform_fee: Uint128,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}
