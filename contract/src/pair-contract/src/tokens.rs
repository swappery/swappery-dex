//! Implementation of total supply.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef, U256};

use crate::helpers::{get_uref};

#[inline]
pub(crate) fn token_address_uref(key_name: &str) -> URef {
    get_uref(key_name)
}

/// Reads a total supply from a specified [`URef`].
pub(crate) fn read_token_address_from(uref: URef) -> U256 {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

/// Writes a total supply to a specific [`URef`].
pub(crate) fn write_token_address_to(uref: URef, value: Address) {
    storage::write(uref, value);
}
