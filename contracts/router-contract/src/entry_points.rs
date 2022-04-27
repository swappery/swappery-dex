use alloc::{string::String, vec, vec::Vec};

use casper_erc20::{ Address };

use casper_types::{
    U256, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
    ContractHash, 
};

use crate::constants::{
    CREATE_PAIR_ENTRY_POINT, GET_PAIR_ENTRY_POINT, SET_FEETO_ENTRY_POINT, SET_FEETO_SETTER_ENTRY_POINT,
    TOKEN0_RUNTIME_ARG_NAME, TOKEN1_RUNTIME_ARG_NAME, ADD_LIQUIDITY_ENTRY_POINT_NAME, 
    AMOUNT0_DESIRED_RUNTIME_ARG_NAME, AMOUNT1_DESIRED_RUNTIME_ARG_NAME, AMOUNT0_MIN_RUNTIME_ARG_NAME,
    AMOUNT1_MIN_RUNTIME_ARG_NAME, TO_RUNTIME_ARG_NAME, DEAD_LINE_RUNTIME_ARG_NAME,  LIQUIDITY_RUNTIME_ARG_NAME,
    REMOVE_LIQUIDITY_ENTRY_POINT_NAME, SWAP_EXACT_TOKENS_FOR_TOKENS_ENTRY_POINT_NAME, AMOUNT_IN_RUNTIME_ARG_NAME,
    AMOUNT_OUT_RUNTIME_ARG_NAME, AMOUNT_IN_MAX_RUNTIME_ARG_NAME, AMOUNT_OUT_MIN_RUNTIME_ARG_NAME,
    PATH_RUNTIME_ARG_NAME, SWAP_TOKENS_FOR_EXACT_TOKENS_ENTRY_POINT_NAME, FEETO_SETTER_KEY_NAME,
    FEETO_KEY_NAME, PAIR_RUNTIME_ARG_NAME
};

pub fn create_pair() -> EntryPoint {
    EntryPoint::new(
        String::from(CREATE_PAIR_ENTRY_POINT),
        vec![
            Parameter::new(TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(PAIR_RUNTIME_ARG_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_pair() -> EntryPoint {
    EntryPoint::new(
        String::from(GET_PAIR_ENTRY_POINT),
        vec![
            Parameter::new(TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_feeto() -> EntryPoint {
    EntryPoint::new(
        String::from(SET_FEETO_ENTRY_POINT),
        vec![
            Parameter::new(FEETO_KEY_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_feeto_setter() -> EntryPoint {
    EntryPoint::new(
        String::from(SET_FEETO_SETTER_ENTRY_POINT),
        vec![
            Parameter::new(FEETO_SETTER_KEY_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn add_liquidity() -> EntryPoint {
    EntryPoint::new(
        String::from(ADD_LIQUIDITY_ENTRY_POINT_NAME),
        vec![
            Parameter::new(TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(AMOUNT0_DESIRED_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT1_DESIRED_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT0_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT1_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn remove_liquidity() -> EntryPoint {
    EntryPoint::new(
        String::from(REMOVE_LIQUIDITY_ENTRY_POINT_NAME),
        vec![
            Parameter::new(TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(LIQUIDITY_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT0_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT1_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn swap_exact_tokens_for_tokens() -> EntryPoint {
    EntryPoint::new(
        String::from(SWAP_EXACT_TOKENS_FOR_TOKENS_ENTRY_POINT_NAME),
        vec![
            Parameter::new(AMOUNT_IN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT_OUT_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(PATH_RUNTIME_ARG_NAME, Vec::<ContractHash>::cl_type()),
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn swap_tokens_for_exact_tokens() -> EntryPoint {
    EntryPoint::new(
        String::from(SWAP_TOKENS_FOR_EXACT_TOKENS_ENTRY_POINT_NAME),
        vec![
            Parameter::new(AMOUNT_OUT_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT_IN_MAX_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(PATH_RUNTIME_ARG_NAME, Vec::<ContractHash>::cl_type()),
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn default() -> EntryPoints {
    let mut contract_entry_points = EntryPoints::new();
    contract_entry_points.add_entry_point(create_pair());
    contract_entry_points.add_entry_point(get_pair());
    contract_entry_points.add_entry_point(set_feeto());
    contract_entry_points.add_entry_point(set_feeto_setter());
    contract_entry_points.add_entry_point(add_liquidity());
    contract_entry_points.add_entry_point(remove_liquidity());
    contract_entry_points.add_entry_point(swap_exact_tokens_for_tokens());
    contract_entry_points.add_entry_point(swap_tokens_for_exact_tokens());
    contract_entry_points
}