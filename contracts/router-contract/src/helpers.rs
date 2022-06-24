//! Implementation details.
use core::convert::TryInto;

extern crate alloc;

use alloc::{collections::BTreeMap, format, string::ToString, vec::Vec};

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::FromBytes, runtime_args, system::CallStackElement, ApiError, CLTyped, ContractHash,
    ContractPackageHash, RuntimeArgs, URef, U256,
};

use casper_erc20::{Address, Error};

use crate::error::Error as RouterError;
use crate::event::RouterEvent;

use crate::constants::GET_RESERVES_ENTRY_POINT_NAME;

pub(crate) fn get_uref(name: &str) -> URef {
    let key = runtime::get_key(name)
        .ok_or(ApiError::MissingKey)
        .unwrap_or_revert();
    key.try_into().unwrap_or_revert()
}

pub(crate) fn read_from<T>(name: &str) -> T
where
    T: FromBytes + CLTyped,
{
    let uref = get_uref(name);
    let value: T = storage::read(uref).unwrap_or_revert().unwrap_or_revert();
    value
}

fn get_immediate_call_stack_item() -> Option<CallStackElement> {
    let call_stack = runtime::get_call_stack();
    call_stack.into_iter().rev().nth(1)
}

fn call_stack_element_to_address(call_stack_element: CallStackElement) -> Address {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Address::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => Address::from(account_hash),
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Address::from(contract_package_hash),
    }
}

pub(crate) fn get_immediate_caller_address() -> Result<Address, Error> {
    get_immediate_call_stack_item()
        .map(call_stack_element_to_address)
        .ok_or(Error::InvalidContext)
}

pub(crate) fn get_caller_address() -> Result<Address, Error> {
    let call_stack = runtime::get_call_stack();
    let top_of_the_stack = call_stack
        .into_iter()
        .rev()
        .next()
        .ok_or(Error::InvalidContext)?;
    let address = call_stack_element_to_address(top_of_the_stack);
    Ok(address)
}

pub(crate) fn quote(amount0: U256, reserve0: U256, reserve1: U256) -> U256 {
    if !(amount0 > U256::zero()) {
        // require(amountA > 0, 'PancakeLibrary: INSUFFICIENT_AMOUNT');
    }
    if !(reserve0 > U256::zero() && reserve1 > U256::zero()) {
        // require(reserveA > 0 && reserveB > 0, 'PancakeLibrary: INSUFFICIENT_LIQUIDITY');
    }
    amount0 * reserve1 / reserve0
}

pub(crate) fn sort_tokens(
    token0: ContractHash,
    token1: ContractHash,
) -> (ContractHash, ContractHash) {
    let tokens: (ContractHash, ContractHash);
    if token0.lt(&token1) {
        tokens = (token0, token1);
    } else {
        tokens = (token1, token0);
    }
    tokens
}

pub(crate) fn get_reserves(
    token0: ContractHash,
    token1: ContractHash,
    pair: Address,
) -> (U256, U256) {
    let _reserves: (U256, U256) = runtime::call_versioned_contract(
        *pair.as_contract_package_hash().unwrap_or_revert(),
        None,
        GET_RESERVES_ENTRY_POINT_NAME,
        runtime_args! {},
    );
    let (tokena, ..) = sort_tokens(token0, token1);
    let reserves = if tokena.eq(&token0) {
        _reserves
    } else {
        (_reserves.1, _reserves.0)
    };
    reserves
}

pub(crate) fn get_amount_out(amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    if !(amount_in > U256::zero()) {
        runtime::revert(RouterError::InsufficientInputAmount);
    }
    if !(reserve_in > U256::zero() && reserve_out > U256::zero()) {
        runtime::revert(RouterError::InsufficientLiquidity);
    }

    let amount_with_fee: U256 = amount_in * U256::from(998u64);
    let nume: U256 = amount_with_fee * reserve_out;
    let deno: U256 = reserve_in * U256::from(1000u64) + amount_with_fee;
    nume / deno
}

pub(crate) fn get_amount_in(amount_out: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    if !(amount_out > U256::zero()) {
        runtime::revert(RouterError::InsufficientOutputAmount);
    }
    if !(reserve_in > U256::zero() && reserve_out > U256::zero()) {
        runtime::revert(RouterError::InsufficientLiquidity);
    }

    let nume: U256 = reserve_in * amount_out * U256::from(1000u64);
    let deno: U256 = (reserve_out - amount_out) * U256::from(998u64);
    (nume / deno) + U256::one()
}

pub fn contract_package_hash() -> ContractPackageHash {
    let call_stacks = runtime::get_call_stack();
    let last_entry = call_stacks.last().unwrap_or_revert();
    let package_hash: Option<ContractPackageHash> = match last_entry {
        CallStackElement::StoredContract {
            contract_package_hash,
            contract_hash: _,
        } => Some(*contract_package_hash),
        _ => None,
    };
    package_hash.unwrap_or_revert()
}

pub fn emit(event: &RouterEvent) {
    let mut events = Vec::new();
    match event {
        RouterEvent::CreatePair {
            token0,
            token1,
            pair,
        } => {
            let mut param = BTreeMap::new();
            param.insert(
                "contract_package_hash",
                contract_package_hash().to_formatted_string(),
            );
            param.insert("event_type", "create_pair".to_string());
            param.insert("token0", token0.to_string());
            param.insert("token1", token1.to_string());
            param.insert("pair", pair.to_string());
            events.push(param);
        }
        RouterEvent::AddLiquidity {
            token0,
            token1,
            amount0,
            amount1,
            recipient,
        } => {
            let mut param = BTreeMap::new();
            param.insert(
                "contract_package_hash",
                contract_package_hash().to_formatted_string(),
            );
            param.insert("event_type", "add_liquidity".to_string());
            param.insert("token0", token0.to_string());
            param.insert("token1", token1.to_string());
            param.insert("amount0", amount0.to_string());
            param.insert("amount1", amount1.to_string());
            param.insert("recipient", recipient.to_string());
            events.push(param);
        }
        RouterEvent::RemoveLiquidity {
            token0,
            token1,
            liquidity,
            recipient,
        } => {
            let mut param = BTreeMap::new();
            param.insert(
                "contract_package_hash",
                contract_package_hash().to_formatted_string(),
            );
            param.insert("event_type", "remove_liquidity".to_string());
            param.insert("token0", token0.to_string());
            param.insert("token1", token1.to_string());
            param.insert("liquidity", liquidity.to_string());
            param.insert("recipient", recipient.to_string());
            events.push(param);
        }
        RouterEvent::SwapExactIn {
            amount_in,
            amount_out,
            path,
            recipient,
        } => {
            let mut param = BTreeMap::new();
            param.insert(
                "contract_package_hash",
                contract_package_hash().to_formatted_string(),
            );
            param.insert("event_type", "swap_exact_in".to_string());
            param.insert("amount_in", amount_in.to_string());
            param.insert("amount_out", amount_out.to_string());
            param.insert("path", format!("{:?}", path));
            param.insert("recipient", recipient.to_string());
            events.push(param);
        }
        RouterEvent::SwapExactOut {
            amount_in,
            amount_out,
            path,
            recipient,
        } => {
            let mut param = BTreeMap::new();
            param.insert(
                "contract_package_hash",
                contract_package_hash().to_formatted_string(),
            );
            param.insert("event_type", "swap_exact_out".to_string());
            param.insert("amount_in", amount_in.to_string());
            param.insert("amount_out", amount_out.to_string());
            param.insert("path", format!("{:?}", path));
            param.insert("recipient", recipient.to_string());
            events.push(param);
        }
        RouterEvent::Installed { contract_hash } => {
            let mut param = BTreeMap::new();
            param.insert("event_type", "installed".to_string());
            param.insert("contract_hash", contract_hash.to_string());
            events.push(param);
        }
    };
    for param in events {
        let _: URef = storage::new_uref(param);
    }
}
