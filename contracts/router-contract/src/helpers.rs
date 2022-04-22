//! Implementation details.
use core::convert::TryInto;

extern crate std;

use std::cmp::Ordering;

use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{bytesrepr::FromBytes, system::CallStackElement, ApiError, CLTyped, URef, U256,
    ContractHash,
};

use casper_erc20::{Error, Address};

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

pub(crate) fn sort_addresses(token0: Address, token1: Address) -> (Address, Address) {
    let mut addresses: (Address, Address);
    if token0.as_contract_package_hash().unwrap_or_revert().eq(token1.as_contract_package_hash().unwrap_or_revert()) {
        //
    }
    if token0.as_contract_package_hash().unwrap_or_revert().lt(token1.as_contract_package_hash().unwrap_or_revert()) {
        addresses = (token0, token1);
    } else {
        addresses = (token1, token0);
    }
    addresses
}