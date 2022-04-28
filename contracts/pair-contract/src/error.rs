use casper_types::ApiError;

pub enum Error {
    InsufficientInputAmount,
    InsufficientOutputAmount,
    InsufficientLiquidity,
    InsufficientLiquidityBurned,
    InsufficientLiquidityMinted,
    InvalidTo,
    OverFlow,
    Forbidden,
    Locked,
    K,
    InvalidContext,
    InsufficientBalance,
    InsufficientAllowance,
}

const ERROR_INSUFFICIENT_INPUT_AMOUNT: u16 = u16::MAX - 4;
const ERROR_INSUFFICIENT_OUTPUT_AMOUNT: u16 = u16::MAX - 5;
const ERROR_INSUFFICIENT_LIQUIDITY: u16 = u16::MAX - 6;
const ERROR_INSUFFICIENT_LIQUIDITY_BURNED: u16 = u16::MAX - 7;
const ERROR_INSUFFICIENT_LIQUIDITY_MINTED: u16 = u16::MAX - 8;
const ERROR_INVALID_TO: u16 = u16::MAX - 9;
const ERROR_OVERFLOW: u16 = u16::MAX - 10;
const ERROR_FORBIDDEN: u16 = u16::MAX - 11;
const ERROR_LOCKED: u16 = u16::MAX - 12;
const ERROR_K: u16 = u16::MAX - 13;
const ERROR_INVALID_CONTEXT: u16 = u16::MAX - 14;
const ERROR_INSUFFICIENT_BALANCE: u16 = u16::MAX - 15;
const ERROR_INSUFFICIENT_ALLOWANCE: u16 = u16::MAX - 16;

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        let user_error = match error {
            Error::InsufficientInputAmount => ERROR_INSUFFICIENT_INPUT_AMOUNT,
            Error::InsufficientOutputAmount => ERROR_INSUFFICIENT_OUTPUT_AMOUNT,
            Error::InsufficientLiquidity => ERROR_INSUFFICIENT_LIQUIDITY,
            Error::InsufficientLiquidityBurned => ERROR_INSUFFICIENT_LIQUIDITY_BURNED,
            Error::InsufficientLiquidityMinted => ERROR_INSUFFICIENT_LIQUIDITY_MINTED,
            Error::InvalidTo => ERROR_INVALID_TO,
            Error::OverFlow => ERROR_OVERFLOW,
            Error::Forbidden => ERROR_FORBIDDEN,
            Error::Locked => ERROR_LOCKED,
            Error::K => ERROR_K,
            Error::InvalidContext => ERROR_INVALID_CONTEXT,
            Error::InsufficientBalance => ERROR_INSUFFICIENT_BALANCE,
            Error::InsufficientAllowance => ERROR_INSUFFICIENT_ALLOWANCE,
        };
        ApiError::User(user_error)
    }
}
