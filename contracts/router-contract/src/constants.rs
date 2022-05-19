//entry point names
pub const CREATE_PAIR_ENTRY_POINT: &str = "create_pair";
pub const GET_PAIR_ENTRY_POINT: &str = "get_pair";
pub const SET_FEETO_ENTRY_POINT: &str = "set_feeto";
pub const SET_FEETO_SETTER_ENTRY_POINT: &str = "set_feeto_setter";
pub const MINT_ENTRY_POINT_NAME: &str = "mint";
pub const BURN_ENTRY_POINT_NAME: &str = "burn";
pub const SWAP_ENTRY_POINT_NAME: &str = "swap";
pub const GET_RESERVES_ENTRY_POINT_NAME: &str = "get_reserves";
pub const ADD_LIQUIDITY_ENTRY_POINT_NAME: &str = "add_liquidity";
pub const REMOVE_LIQUIDITY_ENTRY_POINT_NAME: &str = "remove_liquidity";
pub const SWAP_EXACT_TOKENS_FOR_TOKENS_ENTRY_POINT_NAME: &str = "swap_exact_tokens_for_tokens";
pub const SWAP_TOKENS_FOR_EXACT_TOKENS_ENTRY_POINT_NAME: &str = "swap_tokens_for_exact_tokens";
pub const BALANCE_OF_ENTRY_POINT_NAME: &str = "balance_of";
pub const SWAP_EXACT_TOKENS_FOR_TOKENS_SUPPORTING_FEE_ENTRY_POINT_NAME: &str =
    "swap_exact_tokens_for_tokens_supporting_fee";
pub const WITHDRAW_ENTRY_POINT_NAME: &str = "withdraw";

//runtime args names
pub const TOKEN0_RUNTIME_ARG_NAME: &str = "token0";
pub const TOKEN1_RUNTIME_ARG_NAME: &str = "token1";
pub const PAIR_RUNTIME_ARG_NAME: &str = "pair";
pub const PAIR_CONTRACT_RUNTIME_ARG_NAME: &str = "pair_contract";
pub const AMOUNT0_DESIRED_RUNTIME_ARG_NAME: &str = "amount0_desired";
pub const AMOUNT1_DESIRED_RUNTIME_ARG_NAME: &str = "amount1_desired";
pub const AMOUNT0_MIN_RUNTIME_ARG_NAME: &str = "amount0_min";
pub const AMOUNT1_MIN_RUNTIME_ARG_NAME: &str = "amount1_min";
pub const TO_RUNTIME_ARG_NAME: &str = "to";
pub const DEAD_LINE_RUNTIME_ARG_NAME: &str = "dead_line";
pub const LIQUIDITY_RUNTIME_ARG_NAME: &str = "liquidity";
pub const AMOUNT0_RUNTIME_ARG_NAME: &str = "amount0";
pub const AMOUNT1_RUNTIME_ARG_NAME: &str = "amount1";
pub const AMOUNT_IN_RUNTIME_ARG_NAME: &str = "amount_in";
pub const AMOUNT_OUT_RUNTIME_ARG_NAME: &str = "amount_out";
pub const AMOUNT_IN_MAX_RUNTIME_ARG_NAME: &str = "amount_in_max";
pub const AMOUNT_OUT_MIN_RUNTIME_ARG_NAME: &str = "amount_out_min";
pub const PATH_RUNTIME_ARG_NAME: &str = "path";
pub const CONTRACT_KEY_NAME_ARG_NAME: &str = "contract_key_name";
pub const ADDRESS_RUNTIME_ARG_NAME: &str = "address";
pub const AMOUNT_RUNTIME_ARG_NAME: &str = "amount";

//key names
pub const FEETO_KEY_NAME: &str = "feeto";
pub const FEETO_SETTER_KEY_NAME: &str = "feeto_setter";
pub const PAIR_LIST_KEY_NAME: &str = "pair_list";
pub const PAIR_CONTRACT_LIST_KEY_NAME: &str = "pair_contract_list";
pub const WCSPR_CONTRACT_KEY_NAME: &str = "wcspr_token";
