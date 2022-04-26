use casper_types::ApiError;

pub enum Error {
    ExcessiveInputAmount,
    InsufficientOutputAmount,
    InsufficientInputAmount,
    InsufficientAAmount,
    InsufficientBAmount,
    InsufficientLiquidity,
    InvalidPath,
}

const ERROR_EXCESSIVE_INPUT_AMOUNT: u16 = u16::MAX;
const ERROR_INSUFFICIENT_OUTPUT_AMOUNT: u16 = u16::MAX-1;
const ERROR_INSUFFICIENT_INPUT_AMOUNT: u16 = u16::MAX-2;
const ERROR_INSUFFICIENT_A_AMOUNT: u16 = u16::MAX - 3;
const ERROR_INSUFFICIENT_B_AMOUNT: u16 = u16::MAX - 4;
const ERROR_INSUFFICIENT_LIQUIDITY: u16 = u16::MAX - 5;
const ERROR_INVALID_PATH: u16 = u16::MAX - 5;

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
        };
        ApiError::User(user_error)
    }
}