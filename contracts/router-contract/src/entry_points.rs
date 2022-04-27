use alloc::{string::String, vec, vec::Vec};

use casper_erc20::{ Address };

use casper_types::{
    U256, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
    ContractHash, 
};

use crate::constants as consts;

pub fn create_pair() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::CREATE_PAIR_ENTRY_POINT),
        vec![
            Parameter::new(consts::TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::PAIR_RUNTIME_ARG_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn get_pair() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::GET_PAIR_ENTRY_POINT),
        vec![
            Parameter::new(consts::TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_feeto() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::SET_FEETO_ENTRY_POINT),
        vec![
            Parameter::new(consts::FEETO_KEY_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn set_feeto_setter() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::SET_FEETO_SETTER_ENTRY_POINT),
        vec![
            Parameter::new(consts::FEETO_SETTER_KEY_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn add_liquidity() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::ADD_LIQUIDITY_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::AMOUNT0_DESIRED_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT1_DESIRED_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT0_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT1_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn remove_liquidity() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::REMOVE_LIQUIDITY_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::TOKEN0_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::TOKEN1_RUNTIME_ARG_NAME, ContractHash::cl_type()),
            Parameter::new(consts::LIQUIDITY_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT0_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT1_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn swap_exact_tokens_for_tokens() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::SWAP_EXACT_TOKENS_FOR_TOKENS_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::AMOUNT_IN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT_OUT_MIN_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::PATH_RUNTIME_ARG_NAME, Vec::<ContractHash>::cl_type()),
            Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn swap_tokens_for_exact_tokens() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::SWAP_TOKENS_FOR_EXACT_TOKENS_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::AMOUNT_OUT_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT_IN_MAX_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::PATH_RUNTIME_ARG_NAME, Vec::<ContractHash>::cl_type()),
            Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::DEAD_LINE_RUNTIME_ARG_NAME, U256::cl_type()),            
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