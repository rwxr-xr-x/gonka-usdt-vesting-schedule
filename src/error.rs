use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("caller is not the governor")]
    NotGovernor {},

    #[error("contract is frozen")]
    ContractFrozen {},

    #[error("tranche {id} does not exist")]
    TrancheNotFound { id: u8 },

    #[error("tranche {id} has already been released")]
    TrancheAlreadyReleased { id: u8 },

    #[error("tranche {id} matures at {matures_at}, current time is {now}")]
    TrancheNotYetMature { id: u8, matures_at: u64, now: u64 },

    #[error("contract holds {held} but needs {required}")]
    InsufficientFunds { held: String, required: String },
}
