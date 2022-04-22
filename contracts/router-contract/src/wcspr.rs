//! Implementation of total supply.

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{URef, ContractHash};

use crate::{constants::WCSPR_CONTRACT_KEY_NAME, helpers};

#[inline]
pub(crate) fn wcspr_uref() -> URef {
    helpers::get_uref(WCSPR_CONTRACT_KEY_NAME)
}

pub(crate) fn read_wcspr_from(uref: URef) -> ContractHash {
    storage::read(uref).unwrap_or_revert().unwrap_or_revert()
}

pub(crate) fn write_wcspr_to(uref: URef, value: ContractHash) {
    storage::write(uref, value);
}
