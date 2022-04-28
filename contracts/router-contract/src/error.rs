use casper_types::ApiError;

pub enum Error {
    ExcessiveInputAmount,
    InsufficientOutputAmount,
    InsufficientInputAmount,
    InsufficientAAmount,
    InsufficientBAmount,
    InsufficientLiquidity,
    InvalidPath,
    Expired,
    Permission,
}

const ERROR_EXCESSIVE_INPUT_AMOUNT: u16 = u16::MAX - 17;
const ERROR_INSUFFICIENT_OUTPUT_AMOUNT: u16 = u16::MAX - 18;
const ERROR_INSUFFICIENT_INPUT_AMOUNT: u16 = u16::MAX - 19;
const ERROR_INSUFFICIENT_A_AMOUNT: u16 = u16::MAX - 20;
const ERROR_INSUFFICIENT_B_AMOUNT: u16 = u16::MAX - 21;
const ERROR_INSUFFICIENT_LIQUIDITY: u16 = u16::MAX - 22;
const ERROR_INVALID_PATH: u16 = u16::MAX - 23;
const ERROR_EXPIRED: u16 = u16::MAX - 24;
const ERROR_PERMISSION: u16 = u16::MAX - 25;

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        let user_error = match error {
            Error::ExcessiveInputAmount => ERROR_EXCESSIVE_INPUT_AMOUNT,
            Error::InsufficientOutputAmount => ERROR_INSUFFICIENT_OUTPUT_AMOUNT,
            Error::InsufficientInputAmount => ERROR_INSUFFICIENT_INPUT_AMOUNT,
            Error::InsufficientAAmount => ERROR_INSUFFICIENT_A_AMOUNT,
            Error::InsufficientBAmount => ERROR_INSUFFICIENT_B_AMOUNT,
            Error::InsufficientLiquidity => ERROR_INSUFFICIENT_LIQUIDITY,
            Error::InvalidPath => ERROR_INVALID_PATH,
            Error::Expired => ERROR_EXPIRED,
            Error::Permission => ERROR_PERMISSION,
        };
        ApiError::User(user_error)
    }
}
