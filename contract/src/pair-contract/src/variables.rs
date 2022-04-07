//! Implementation of total supply.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef, U256};
use casper_erc20::Address;

use crate::{constants::{
    RESERVE0_KEY_NAME, RESERVE1_KEY_NAME, TOKEN0_KEY_NAME, TOKEN1_KEY_NAME, LOCKED_FLAG_KEY_NAME,
}, helpers};

#[inline]
pub(crate) fn reserve0_uref() -> URef {
    helpers::get_uref(RESERVE0_KEY_NAME)
}

#[inline]
pub(crate) fn reserve1_uref() -> URef {
    helpers::get_uref(RESERVE1_KEY_NAME)
}

#[inline]
pub(crate) fn token0_uref() -> URef {
    helpers::get_uref(TOKEN0_KEY_NAME)
}

#[inline]
pub(crate) fn token1_uref() -> URef {
    helpers::get_uref(TOKEN1_KEY_NAME)
}

#[inline]
pub(crate) fn locked_uref() -> URef {
    helpers::get_uref(LOCKED_FLAG_KEY_NAME)
}

pub(crate) fn read_reserve_from(uref: URef) -> U256 {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_reserve_to(uref: URef, value: U256) {
    storage::write(uref, value);
}

pub(crate) fn read_token_from(uref: URef) -> Address {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_token_to(uref: URef, value: Address) {
    storage::write(uref, value);
}

pub(crate) fn read_locked_from(uref: URef) -> bool {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_locked_to(uref: URef, value: bool) {
    storage::write(uref, value);
}