use casper_types::U256;

pub const TO_RUNTIME_ARG_NAME: &str = "to";
pub const AMOUNT0_RUNTIME_ARG_NAME: &str = "amount0";
pub const AMOUNT1_RUNTIME_ARG_NAME: &str = "amount1";
pub const RESERVE0_KEY_NAME: &str = "reserve0";
pub const RESERVE1_KEY_NAME: &str = "reserve1";
pub const TOKEN0_KEY_NAME: &str = "token0";
pub const TOKEN1_KEY_NAME: &str = "token1";
pub const KLAST_KEY_NAME: &str = "klast";
pub const FACTORY_KEY_NAME: &str = "factory";
pub const LOCKED_FLAG_KEY_NAME: &str = "locked";
pub const MINT_ENTRY_POINT_NAME: &str = "mint";
pub const BURN_ENTRY_POINT_NAME: &str = "burn";
pub const SWAP_ENTRY_POINT_NAME: &str = "swap";
pub const GET_RESERVES_ENTRY_POINT_NAME: &str = "get_reserves";
pub const MINIMUM_LIQUIDITY: U256 = U256::from("1000");