use alloc::{string::String, vec, vec::Vec};

use casper_types::{
    CLType, CLTyped, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Parameter, U256,
};

use crate::address::Address;
use crate::constants as consts;

/// Returns the `name` entry point.
pub fn name() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::NAME_ENTRY_POINT_NAME),
        Vec::new(),
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `symbol` entry point.
pub fn symbol() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::SYMBOL_ENTRY_POINT_NAME),
        Vec::new(),
        String::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}
/// Returns the `transfer_from` entry point.
pub fn transfer_from() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::TRANSFER_FROM_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::OWNER_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::RECIPIENT_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `allowance` entry point.
pub fn allowance() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::ALLOWANCE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::OWNER_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::SPENDER_RUNTIME_ARG_NAME, Address::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `approve` entry point.
pub fn approve() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::APPROVE_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::SPENDER_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `transfer` entry point.
pub fn transfer() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::TRANSFER_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::RECIPIENT_RUNTIME_ARG_NAME, Address::cl_type()),
            Parameter::new(consts::AMOUNT_RUNTIME_ARG_NAME, U256::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `balance_of` entry point.
pub fn balance_of() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::BALANCE_OF_ENTRY_POINT_NAME),
        vec![Parameter::new(consts::ADDRESS_RUNTIME_ARG_NAME, Address::cl_type())],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `total_supply` entry point.
pub fn total_supply() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::TOTAL_SUPPLY_ENTRY_POINT_NAME),
        Vec::new(),
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the `decimals` entry point.
pub fn decimals() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::DECIMALS_ENTRY_POINT_NAME),
        Vec::new(),
        u8::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the 'mint' entry point.
pub fn mint() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::MINT_ENTRY_POINT_NAME),
        vec![Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the 'burn' entry point.
pub fn burn() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::BURN_ENTRY_POINT_NAME),
        vec![Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type())],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the 'swap' entry point.
pub fn swap() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::SWAP_ENTRY_POINT_NAME),
        vec![
            Parameter::new(consts::AMOUNT0_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::AMOUNT1_RUNTIME_ARG_NAME, U256::cl_type()),
            Parameter::new(consts::TO_RUNTIME_ARG_NAME, Address::cl_type()),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the 'get_reserves' entry point.
pub fn get_reserves() -> EntryPoint {
    EntryPoint::new(
        String::from(consts::GET_RESERVES_ENTRY_POINT_NAME),
        vec![],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    )
}

/// Returns the default entry points of LP token.
pub fn default() -> EntryPoints {
    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(name());
    entry_points.add_entry_point(symbol());
    entry_points.add_entry_point(decimals());
    entry_points.add_entry_point(total_supply());
    entry_points.add_entry_point(balance_of());
    entry_points.add_entry_point(transfer());
    entry_points.add_entry_point(approve());
    entry_points.add_entry_point(allowance());
    entry_points.add_entry_point(transfer_from());
    entry_points.add_entry_point(mint());
    entry_points.add_entry_point(burn());
    entry_points.add_entry_point(swap());
    entry_points.add_entry_point(get_reserves());
    entry_points
}
