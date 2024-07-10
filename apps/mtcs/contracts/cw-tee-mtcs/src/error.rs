use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use hex::FromHexError;
use k256::ecdsa::Error as K256Error;
use quartz_cw::error::Error as QuartzError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Quartz(#[from] QuartzError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Duplicate entry found")]
    DuplicateEntry,

    #[error("Not Secp256K1")]
    K256(K256Error),

    #[error("Invalid hex")]
    Hex(#[from] FromHexError),

    #[error("Invalid length")]
    BadLength,

    #[error("Cw20 error: {0}")]
    Cw20(Cw20ContractError),
}

impl From<K256Error> for ContractError {
    fn from(e: K256Error) -> Self {
        Self::K256(e)
    }
}

impl From<Cw20ContractError> for ContractError {
    fn from(e: Cw20ContractError) -> Self {
        Self::Cw20(e)
    }
}