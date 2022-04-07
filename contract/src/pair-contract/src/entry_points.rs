use alloc::{string::String, vec};

use casper_erc20::{ entry_points, Address };

use casper_types::{
    U256, CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter,
};

use crate::constants::{ TO_RUNTIME_ARG_NAME, AMOUNT0_RUNTIME_ARG_NAME, AMOUNT1_RUNTIME_ARG_NAME};

pub fn mint() -> EntryPoint {
    EntryPoint::new(
        String::from("mint"),
        vec![
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type()),
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
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type())
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

pub fn swap() -> EntryPoint {
    EntryPoint::new(
        String::from("swap"),
        vec![
            Parameter::new(AMOUNT0_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(AMOUNT1_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(TO_RUNTIME_ARG_NAME, Address::cl_type()),
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
    contract_entry_points.add_entry_point(swap());
    contract_entry_points
}