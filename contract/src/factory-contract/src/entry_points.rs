use alloc::{string::String, vec};

use casper_erc20::{ entry_points, Address };

use casper_types::{
    U256, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

use crate::constants::{
    CREATE_PAIR_ENTRY_POINT, GET_PAIR_ENTRY_POINT, SET_FEETO_ENTRY_POINT, SET_FEETO_SETTER_ENTRY_POINT,
    TOKEN0_RUNTIME_ARG_NAME, TOKEN1_RUNTIME_ARG_NAME,
}

pub fn create_pair() -> EntryPoint {
    EntryPoint::new(
        String::from(CREATE_PAIR_ENTRY_POINT),
        vec![
            Parameter::new(TOKEN0_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(TOKEN1_RUNTIME_ARG_NAME, Address::cl_type()),
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
            Parameter::new(TOKEN0_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(TOKEN1_RUNTIME_ARG_NAME, Address::cl_type()),
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

pub fn default() -> EntryPoints {
    let mut contract_entry_points = EntryPoints::new();
    contract_entry_points.add_entry_point(create_pair());
    contract_entry_points.add_entry_point(get_pair());
    contract_entry_points.add_entry_point(set_feeto());
    contract_entry_points.add_entry_point(set_feeto_setter());
    contract_entry_points
}