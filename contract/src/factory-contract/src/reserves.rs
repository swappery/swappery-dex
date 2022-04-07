use alloc::string::String;

use casper_contract::{contract_api::storage, unwrap_or_revert::UnwrapOrRevert};
use casper_types::{bytesrepr::ToBytes, URef, U256};

use casper_erc20::{Error, Address};

use crate::helpers::{get_uref};

const TOKEN0_RESERVE_KEY_NAME: &str = "reserve0";
const TOKEN1_RESERVE_KEY_NAME: &str = "reserve1";

pub(crate) fn get_reserve0_uref() -> URef {
    get_uref(TOKEN0_RESERVE_KEY_NAME)
}

pub(crate) fn get_reserve1_uref() -> URef {
    get_uref(TOKEN1_RESERVE_KEY_NAME)
}

fn make_dictionary_item_key(token0: Address, token1: Address) -> String {
    let mut preimage = Vec::new();
    preimage.append(&mut token0.to_bytes().unwrap_or_revert());
    preimage.append(&mut token1.to_bytes().unwrap_or_revert());

    let key_bytes = runtime::blake2b(&preimage);
    hex::encode(&key_bytes)
}

pub(crate) fn write_reserve_for(
    reserve_uref: URef,
    token0: Address,
    token1: Address,
    amount: U256,
) {
    let dictionary_item_key = make_dictionary_item_key(token0, token1);
    storage::dictionary_put(reserve_uref, &dictionary_item_key, amount)
}

pub(crate) fn read_reserve_from(reserve_uref: URef, token0: Address, token1: Address) -> U256 {
    let dictionary_item_key = make_dictionary_item_key(token0, token1);
    storage::dictionary_get(reserve_uref, &dictionary_item_key)
        .unwrap_or_revert()
        .unwrap_or_default()
}
