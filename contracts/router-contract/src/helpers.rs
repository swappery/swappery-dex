//! Implementation details.
use core::convert::TryInto;

extern crate alloc;

use alloc::vec::Vec;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{bytesrepr::FromBytes, system::CallStackElement, ApiError, CLTyped, URef, U256,
    ContractHash, runtime_args, RuntimeArgs,
};

use casper_erc20::{Error, Address};

use crate::constants::{GET_RESERVES_ENTRY_POINT_NAME};

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
        CallStackElement::StoredSession { account_hash, .. } => {
            Address::from(account_hash)
        }
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

pub(crate) fn sort_tokens(token0: ContractHash, token1: ContractHash) -> (ContractHash, ContractHash) {
    let mut tokens: (ContractHash, ContractHash);
    if token0.eq(&token1) {
        //
    }
    if token0.lt(&token1) {
        tokens = (token0, token1);
    } else{
        tokens = (token1, token0);
    }
    tokens
}

pub(crate) fn get_amount_out(amount_in: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    // require(amountIn > 0, 'PancakeLibrary: INSUFFICIENT_INPUT_AMOUNT');
    // require(reserveIn > 0 && reserveOut > 0, 'PancakeLibrary: INSUFFICIENT_LIQUIDITY');

    let amount_with_fee: U256 = amount_in * U256::from(998u64);
    let nume: U256 = amount_with_fee * reserve_out;
    let deno: U256 = reserve_in * U256::from(1000u64) + amount_with_fee;
    nume / deno
}

pub(crate) fn get_amount_in(amount_out: U256, reserve_in: U256, reserve_out: U256) -> U256 {
    // require(amountOut > 0, 'PancakeLibrary: INSUFFICIENT_OUTPUT_AMOUNT');
    // require(reserveIn > 0 && reserveOut > 0, 'PancakeLibrary: INSUFFICIENT_LIQUIDITY');

    let nume: U256 = reserve_in * amount_out * U256::from(1000u64);
    let deno: U256 = (reserve_out - amount_out) * U256::from(998u64);
    (nume / deno) + U256::one()
}

pub(crate) fn get_amounts_out(amount_in: U256, path: Vec<Address>) -> Vec<U256> {
    // require(path.length >= 2, 'PancakeLibrary: INVALID_PATH');

    let mut amounts: Vec<U256> = Vec::with_capacity(path.len());
    amounts.push(amount_in);
    for i in 0..path.len() - 1 {
        let pair: Address = *path.get(i).unwrap_or_revert();
        let reserves: (U256, U256) = runtime::call_versioned_contract(
            *pair.as_contract_package_hash().unwrap_or_revert(),
            None,
            GET_RESERVES_ENTRY_POINT_NAME,
            runtime_args! {},
        );
        amounts.push(get_amount_out(*amounts.get(i).unwrap_or_revert(), reserves.0, reserves.1));
    }
    amounts
}