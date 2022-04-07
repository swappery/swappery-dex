//! Implementation of total supply.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef, U256};

#[inline]
pub(crate) fn reserve_uref(key_name: &str) -> URef {
    detail::get_uref(key_name)
}

/// Reads a total supply from a specified [`URef`].
pub(crate) fn read_reserve_from(uref: URef) -> U256 {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a total supply to a specific [`URef`].
pub(crate) fn write_reserve_to(uref: URef, value: U256) {
    storage::write(uref, value);
}
