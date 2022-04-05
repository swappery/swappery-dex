use alloc::{string::String, vec};

use casper_erc20::{ entry_points, constants::{ OWNER_RUNTIME_ARG_NAME, AMOUNT_RUNTIME_ARG_NAME}, Address };

use casper_types::{
    URef, U256, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

pub fn mint() -> EntryPoint {
    EntryPoint::new(
        String::from("mint"),
        vec![
            Parameter::new(OWNER_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn burn() -> EntryPoint {
    EntryPoint::new(
        String::from("burn"),
        vec![
            Parameter::new(OWNER_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn default() -> EntryPoints {
    let mut contract_entry_points = entry_points::default();
    contract_entry_points.add_entry_point(mint());
    contract_entry_points.add_entry_point(burn());
    contract_entry_points
}