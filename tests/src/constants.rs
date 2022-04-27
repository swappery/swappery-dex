use once_cell::sync::Lazy;
use casper_types::{
    account::AccountHash, PublicKey, SecretKey,
};

//contracts
pub const CONTRACT_ERC20_TOKEN: &str = "erc20_token.wasm";
pub const CONTRACT_SWAPPERY_PAIR: &str = "swappery_pair.wasm";
pub const CONTRACT_ERC20_TEST_CALL: &str = "erc20_test_call.wasm";
pub const CONTRACT_SWAPPERY_ROUTER: &str = "swappery_router.wasm";
pub const CONTRACT_WCSPR_TOKEN: &str = "wcspr.wasm";

//arguments
pub const ARG_NAME: &str = "name";
pub const ARG_SYMBOL: &str = "symbol";
pub const ARG_DECIMALS: &str = "decimals";
pub const ARG_TOTAL_SUPPLY: &str = "total_supply";
pub const ARG_CONTRACT_KEY_NAME: &str = "contract_key_name";
pub const ARG_TOKEN0: &str = "token0";
pub const ARG_TOKEN1: &str = "token1";
pub const ARG_TO: &str = "to";
pub const ARG_AMOUNT0: &str = "amount0";
pub const ARG_AMOUNT1: &str = "amount1";
pub const ARG_AMOUNT: &str = "amount";
pub const ARG_RECIPIENT: &str = "recipient";
pub const ARG_TOKEN_CONTRACT: &str = "token_contract";
pub const ARG_ADDRESS: &str = "address";
pub const ARG_PAIR: &str = "pair";

//key names
pub const FEETO_KEY_NAME: &str = "feeto";
pub const FEETO_SETTER_KEY_NAME: &str = "feeto_setter";
pub const PAIR_LIST_KEY_NAME: &str = "pair_list";
pub const ROUTER_CONTRACT_KEY_NAME: &str = "swappery_router";

pub const PAIR_NAME: &str = "SwapperyPair";
pub const PAIR_SYMBOL: &str = "SWP";
pub const PAIR_DECIMALS: u8 = 8;
pub const PAIR_TOTAL_SUPPLY: u64 = 0;
pub const PAIR_CONTRACT_KEY_NAME: &str = "swappery_pair";
pub const PAIR_CONTRACT_HASH_KEY_NAME: &str = "swappery_pair_contract_hash";

pub const TOKEN0_NAME: &str = "PairTestToken0";
pub const TOKEN0_SYMBOL: &str = "PTT0";
pub const TOKEN0_DECIMALS: u8 = 8;
pub const TOKEN0_TOTAL_SUPPLY: u64 = 1_000_000;
pub const TOKEN0_CONTRACT_KEY_NAME: &str = "token0";
pub const TOKEN0_CONTRACT_HASH_KEY_NAME: &str = "token0_contract_hash";

pub const TOKEN1_NAME: &str = "PairTestToken1";
pub const TOKEN1_SYMBOL: &str = "PTT1";
pub const TOKEN1_DECIMALS: u8 = 8;
pub const TOKEN1_TOTAL_SUPPLY: u64 = 2_000_000;
pub const TOKEN1_CONTRACT_KEY_NAME: &str = "token1";
pub const TOKEN1_CONTRACT_HASH_KEY_NAME: &str = "token1_contract_hash";

pub const WCSPR_NAME: &str = "PairTestToken1";
pub const WCSPR_SYMBOL: &str = "PTT1";
pub const WCSPR_DECIMALS: u8 = 8;
pub const WCSPR_TOTAL_SUPPLY: u64 = 2_000_000;
pub const WCSPR_CONTRACT_KEY_NAME: &str = "wcspr_token";

//methods
pub const METHOD_TRANSFER: &str = "transfer";
pub const METHOD_MINT: &str = "mint";
pub const METHOD_BURN: &str = "burn";
pub const METHOD_SWAP: &str = "swap";
pub const CHECK_BALANCE_OF_ENTRYPOINT: &str = "check_balance_of";
pub const METHOD_TRANSFER_AS_STORED_CONTRACT: &str = "transfer_as_stored_contract";
pub const METHOD_CREATE_PAIR: &str = "create_pair";
pub const METHOD_GET_PAIR: &str = "get_pair";

pub const RESULT_KEY: &str = "result";
pub const ERC20_TEST_CALL_KEY: &str = "erc20_test_call";

//error
pub const ERROR_INSUFFICIENT_LIQUIDITY: u16 = u16::MAX - 2;
pub const ERROR_K: u16 = u16::MAX - 9;

//accounts
pub const ACCOUNT_1_SECRET_KEY: Lazy<SecretKey> =
    Lazy::new(|| SecretKey::secp256k1_from_bytes(&[221u8; 32]).unwrap());
pub const ACCOUNT_1_PUBLIC_KEY: Lazy<PublicKey> =
    Lazy::new(|| PublicKey::from(&*ACCOUNT_1_SECRET_KEY));
pub const ACCOUNT_1_ADDR: Lazy<AccountHash> = Lazy::new(|| ACCOUNT_1_PUBLIC_KEY.to_account_hash());
