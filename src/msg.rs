use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub governor: String,
    pub beneficiary: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ReleaseTranche { tranche_id: u8 },
    Freeze {},
    Unfreeze {},
    SetBeneficiary { address: String },
    WithdrawAll { to: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(TrancheResponse)]
    Tranche { id: u8 },
    #[returns(AllTranchesResponse)]
    AllTranches {},
    #[returns(BalanceResponse)]
    Balance {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub governor: String,
    pub beneficiary: String,
    pub frozen: bool,
    pub created_at: u64,
}

#[cw_serde]
pub struct TrancheResponse {
    pub index: u8,
    pub token_amount: Uint128,
    pub matures_at: u64,
    pub released: bool,
}

#[cw_serde]
pub struct AllTranchesResponse {
    pub tranches: Vec<TrancheResponse>,
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Coin,
}
