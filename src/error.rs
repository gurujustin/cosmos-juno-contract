use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unauthorized Role")]
    UnauthorizedRole {},

    #[error("Escrow expired ")]
    Expired {},

    #[error("Escrow not expired")]
    NotExpired {},

    #[error("Buying Utility Over Slippages")]
    BuyingUtilityOverSlippages {},
    
    #[error("Insufficient amount")]
    InsufficientToken {},

    #[error("Fee more than amount")]
    InsufficientEthToSwap {},

    #[error("Insufficient Output Amount")]
    InsufficientOutputAmount {},

    #[error("Pool And Token Mismatch")]
    PoolAndTokenMismatch {},

    #[error("Native Input Zero")]
    NativeInputZero {},

    #[error("TokenTypeMismatch")]
    TokenTypeMismatch {},

    #[error("Cw20InputZero")]
    Cw20InputZero {},
}
