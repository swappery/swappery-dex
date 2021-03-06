//! Implementation of variables.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef, U256};

use crate::{constants::{
    RESERVE0_KEY_NAME, RESERVE1_KEY_NAME, LOCKED_FLAG_KEY_NAME,
    KLAST_KEY_NAME, TOTAL_SUPPLY_KEY_NAME
}, helpers};

#[inline]
pub(crate) fn total_supply_uref() -> URef {
    helpers::get_uref(TOTAL_SUPPLY_KEY_NAME)
}

#[inline]
pub(crate) fn reserve0_uref() -> URef {
    helpers::get_uref(RESERVE0_KEY_NAME)
}

#[inline]
pub(crate) fn reserve1_uref() -> URef {
    helpers::get_uref(RESERVE1_KEY_NAME)
}

#[inline]
pub(crate) fn locked_uref() -> URef {
    helpers::get_uref(LOCKED_FLAG_KEY_NAME)
}

#[inline]
pub(crate) fn klast_uref() -> URef {
    helpers::get_uref(KLAST_KEY_NAME)
}

pub(crate) fn read_total_supply_from(uref: URef) -> U256 {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_total_supply_to(uref: URef, value: U256) {
    storage::write(uref, value);
}

pub(crate) fn read_reserve_from(uref: URef) -> U256 {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_reserve_to(uref: URef, value: U256) {
    storage::write(uref, value);
}

pub(crate) fn read_locked_from(uref: URef) -> bool {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_locked_to(uref: URef, value: bool) {
    storage::write(uref, value);
}

pub(crate) fn read_klast_from(uref: URef) -> U256 {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_klast_to(uref: URef, value: U256) {
    storage::write(uref, value);
}