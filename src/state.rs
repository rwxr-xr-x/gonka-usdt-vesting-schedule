use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub governor: Addr,
    pub beneficiary: Addr,
    pub frozen: bool,
    pub created_at: Timestamp,
}

#[cw_serde]
pub struct Tranche {
    pub index: u8,
    pub token_amount: Uint128,
    pub matures_at: Timestamp,
    pub released: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TRANCHES: Map<u8, Tranche> = Map::new("tranche");

/// IBC USDT token denom on Gonka chain (6 decimals)
pub const TOKEN_DENOM: &str = "ibc/115F68FBA220A028C6F6ED08EA0C1A9C8C52798B14FB66E6C89D5D8C06A524D4";

pub const TRANCHE_COUNT: u8 = TRANCHE_AMOUNTS.len() as u8;

pub const TRANCHE_AMOUNTS: [u128; 4] = [
    51_000_000_000, // Tranche 0: 51,000 USDT
    15_000_000_000, // Tranche 1: 15,000 USDT
    15_000_000_000, // Tranche 2: 15,000 USDT
    15_000_000_000, // Tranche 3: 15,000 USDT
];

pub const TRANCHE_OFFSETS: [u64; 4] = [
    0, // Tranche 0: immediate
    90 * 24 * 60 * 60, // Tranche 1: +3 months (90 days)
    180 * 24 * 60 * 60, // Tranche 2: +6 months (180 days)
    270 * 24 * 60 * 60, // Tranche 3: +9 months (270 days)
];
