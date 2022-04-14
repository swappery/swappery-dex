//! Implementation of total supply.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef};
use swappery_pair::Address;

use crate::{constants::{
    FEETO_KEY_NAME, FEETO_SETTER_KEY_NAME,
}, helpers};

#[inline]
pub(crate) fn feeto_uref() -> URef {
    helpers::get_uref(FEETO_KEY_NAME)
}

#[inline]
pub(crate) fn feeto_setter_uref() -> URef {
    helpers::get_uref(FEETO_SETTER_KEY_NAME)
}

pub(crate) fn read_feeto_from(uref: URef) -> Address {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_feeto_to(uref: URef, value: Address) {
    storage::write(uref, value);
}

pub(crate) fn read_feeto_setter_from(uref: URef) -> Address {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_feeto_setter_to(uref: URef, value: Address) {
    storage::write(uref, value);
}